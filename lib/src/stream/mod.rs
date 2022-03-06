use crate::bitwise::bitreader::BitReaderImpl;
use crate::bitwise::bitwriter::convert_to_code_pad_to_byte;

use crate::block::block_encoder::crc_as_bytes;

use std::io::Read;
use std::io::Write;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;

use crate::bitwise::bitreader::BitReader;
use crate::bitwise::bitwriter::BitWriter;
use crate::bitwise::bitwriter::BitWriterImpl;
use crate::block::block_decoder::decode_block;
use crate::block::block_encoder::generate_block_data;

use crate::bitwise::Bit;

use super::block::rle::rle;
use super::block::symbol_statistics::EncodingStrategy;

fn stream_footer(crc: u32) -> Vec<Bit> {
    let mut out = vec![];

    out.append(
        &mut [0x17, 0x72, 0x45, 0x38, 0x50, 0x90]
            .iter()
            .flat_map(|x| convert_to_code_pad_to_byte(*x as u8))
            .collect::<Vec<_>>(),
    );
    let mut crc_as_bits = crc_as_bytes(crc)
        .iter()
        .flat_map(|x| convert_to_code_pad_to_byte(*x))
        .collect::<Vec<_>>();
    out.append(&mut crc_as_bits);
    out
}

fn file_header() -> Vec<Bit> {
    let mut out = vec![];
    out.append(&mut convert_to_code_pad_to_byte(b'B'));
    out.append(&mut convert_to_code_pad_to_byte(b'Z'));
    out.append(&mut convert_to_code_pad_to_byte(b'h'));
    out.append(&mut convert_to_code_pad_to_byte(b'9'));
    out
}

type Work = (Vec<u8>, Vec<u8>);
type ComputationResult = (Vec<Bit>, u32);

struct WorkerThread {
    send_work: Sender<Work>,
    receive_result: Receiver<ComputationResult>,
    pending: bool,
}

impl WorkerThread {
    fn spawn(name: &str, encoding_strategy: EncodingStrategy) -> Self {
        let (send_work, receive_work) = channel::<Work>();
        let (send_result, receive_result) = channel::<ComputationResult>();
        let builder = thread::Builder::new().name(name.into());

        builder
            .spawn(move || {
                while let Ok(work) = receive_work.recv() {
                    let (buffer, rle_data) = work;
                    send_result
                        .send(generate_block_data(&buffer, rle_data, encoding_strategy))
                        .unwrap();
                }
            })
            .unwrap();
        WorkerThread {
            send_work,
            receive_result,
            pending: false,
        }
    }

    fn flush_work_buffer(&mut self, mut bit_writer: impl BitWriter, total_crc: &mut u32) {
        let result = self.receive_result.recv().unwrap();

        bit_writer.write_bits(&result.0).unwrap();
        *total_crc = result.1 ^ ((*total_crc << 1) | (*total_crc >> 31));
        self.pending = false;
    }

    fn send_work(&mut self, work_to_send: Work) {
        self.pending = true;
        self.send_work.send(work_to_send).unwrap();
    }
}

/// Encode a stream into a writer. Takes a reader and a writer (i.e. two instances of [std::fs::File]).
/// The number of threads and the encoding strategy can be specified.
pub fn encode_stream(
    mut read: impl Read,
    mut writer: impl Write,
    num_threads: usize,
    encoding_strategy: EncodingStrategy,
) {
    let mut bit_writer = BitWriterImpl::from_writer(&mut writer);
    // 900_000 * 4 / 5 - RLE can blow up 4chars to 5, hence we keep
    // a safety margin of 180,000
    const BLOCK_SIZE: usize = 720_000;
    let mut total_crc: u32 = 0;

    let mut worker_threads = (0..num_threads)
        .map(|num| WorkerThread::spawn(&format!("Thread {}", num), encoding_strategy))
        .collect::<Vec<_>>();

    bit_writer.write_bits(&file_header()).unwrap();

    let mut finalize = false;
    loop {
        if finalize {
            break;
        }
        for worker_thread in worker_threads.iter_mut() {
            let mut buf = vec![];
            if let Ok(size) = read.by_ref().take(BLOCK_SIZE as u64).read_to_end(&mut buf) {
                if size == 0 {
                    finalize = true;
                    break;
                }
            } else {
                break;
            }
            let rle_data = rle(&buf);
            worker_thread.send_work((buf, rle_data));
        }

        for worker_thread in worker_threads.iter_mut() {
            if worker_thread.pending {
                worker_thread.flush_work_buffer(&mut bit_writer, &mut total_crc);
            }
        }
    }

    bit_writer.write_bits(&stream_footer(total_crc)).unwrap();
    bit_writer.finalize().unwrap();
}

fn read_file_header(mut bit_reader: impl BitReader) -> Result<(), ()> {
    let res = bit_reader.read_bytes(4)?;
    match &res[..] {
        [b'B', b'Z', b'h', _] => Ok(()),
        _ => {
            println!("Not a valid bz2 file");
            Err(())
        }
    }
}

#[derive(Debug, PartialEq)]
enum BlockType {
    StreamFooter,
    BlockHeader,
}

fn what_next(mut bit_reader: impl BitReader) -> Result<BlockType, ()> {
    let res = bit_reader.read_bytes(6)?;
    match &res[..] {
        [0x31u8, 0x41u8, 0x59u8, 0x26u8, 0x53u8, 0x59u8] => Ok(BlockType::BlockHeader),
        [0x17, 0x72, 0x45, 0x38, 0x50, 0x90] => Ok(BlockType::StreamFooter),
        _ => {
            println!("Expected block start or stream end");
            Err(())
        }
    }
}

/// Decode a stream into a writer. Takes a reader and a writer (i.e. two instances of [std::fs::File])
pub fn decode_stream(mut reader: impl Read, mut writer: impl Write) -> Result<(), ()> {
    let mut bit_reader = BitReaderImpl::from_reader(&mut reader);
    read_file_header(&mut bit_reader)?;
    loop {
        match what_next(&mut bit_reader)? {
            BlockType::StreamFooter => break,
            BlockType::BlockHeader => {
                decode_block(&mut bit_reader, &mut writer)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {

    use crate::bitwise::bitreader::BitReaderImpl;

    use super::*;
    use std::io::Cursor;

    #[test]
    pub fn accepts_correct_header() {
        let input = b"BZh9";
        let mut cursor = Cursor::new(input);
        let mut bit_reader = BitReaderImpl::from_reader(&mut cursor);
        let read = read_file_header(&mut bit_reader);
        assert!(read.is_ok());
    }

    #[test]
    pub fn detects_block_header() {
        let data = vec![0x31u8, 0x41u8, 0x59u8, 0x26u8, 0x53u8, 0x59u8];
        let mut cursor = Cursor::new(data);
        let mut bit_reader = BitReaderImpl::from_reader(&mut cursor);

        assert_eq!(BlockType::BlockHeader, what_next(&mut bit_reader).unwrap());
    }

    #[test]
    pub fn detects_stream_footer() {
        let data = vec![0x17, 0x72, 0x45, 0x38, 0x50, 0x90];
        let mut cursor = Cursor::new(data);
        let mut bit_reader = BitReaderImpl::from_reader(&mut cursor);

        assert_eq!(BlockType::StreamFooter, what_next(&mut bit_reader).unwrap());
    }

    #[test]
    pub fn detects_error() {
        let data = vec![0, 1, 2, 3, 4, 5];
        let mut cursor = Cursor::new(data);
        let mut bit_reader = BitReaderImpl::from_reader(&mut cursor);

        assert!(what_next(&mut bit_reader).is_err());
    }
}
