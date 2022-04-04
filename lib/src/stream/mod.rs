use crate::bitwise::bitreader::BitReaderImpl;
use crate::bitwise::bitwriter::convert_to_code_pad_to_byte;
use crate::bitwise::bitwriter::BufferBitWriter;

use crate::block::block_encoder::crc_as_bytes;

use std::io::Read;
use std::io::Write;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;

use crate::bitwise::bitreader::BitReader;
use crate::bitwise::bitwriter::BitWriter;
use crate::block::block_decoder::decode_block;
use crate::block::block_encoder::generate_block_data;
use crate::block::crc32::crc32;

use crate::bitwise::Bit;

use super::block::rle::rle;
use super::block::rle::rle_augment;
use super::block::rle::rle_total_size;
use super::block::symbol_statistics::EncodingStrategy;

/// Encoder to bzip2 encode a stream.
/// ```rust
/// use libribzip2::EncodingStrategy;
/// use libribzip2::stream::Encoder;
/// use std::io::Cursor;
///
/// let num_threads = 4;
/// let encoding_strategy = EncodingStrategy::Single;
///
/// let reader = Cursor::new(vec![1, 2, 3, 4]);
/// let mut writer = vec![];
///
/// let mut encoder = Encoder::new(reader, encoding_strategy, num_threads);
/// std::io::copy(&mut encoder, &mut writer).unwrap();
/// ```
pub struct Encoder<T: Read> {
    reader: T,
    num_threads: usize,
    encoding_strategy: EncodingStrategy,
    bit_writer: BufferBitWriter,
    total_crc: u32,
    finalized: bool,
    encoded: bool,
    initialized: bool,
}

impl<T> Encoder<T>
where
    T: Read,
{
    pub fn new(reader: T, encoding_strategy: EncodingStrategy, num_threads: usize) -> Self {
        Encoder {
            reader,
            num_threads,
            encoding_strategy,
            bit_writer: Default::default(),
            total_crc: 0,
            finalized: false,
            encoded: false,
            initialized: false,
        }
    }
}

impl<T> Read for Encoder<T>
where
    T: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        const RLE_LIMIT: usize = 900_000;

        let mut worker_threads = (0..self.num_threads)
            .map(|num| WorkerThread::spawn(&format!("Thread {}", num), self.encoding_strategy))
            .collect::<Vec<_>>();

        if !self.initialized {
            self.bit_writer.write_bits(&file_header()).unwrap();
            self.initialized = true;
        }

        while !self.finalized && !self.encoded {
            if self.bit_writer.content() > buf.len() {
                break;
            }
            for worker_thread in worker_threads.iter_mut() {
                let mut buf = vec![];
                let mut rle_data = vec![];
                let mut rle_total_count = 0;
                let mut rle_count = 0;
                let mut rle_last_char = None;
                while rle_total_count < RLE_LIMIT {
                    // RLE can blow up 4chars to 5, hence we keep a safety margin
                    let to_take = (RLE_LIMIT - rle_data.len()) * 4 / 5;
                    let mut buf_current = vec![];
                    if let Ok(size) = self
                        .reader
                        .by_ref()
                        .take(to_take as u64)
                        .read_to_end(&mut buf_current)
                    {
                        if size == 0 {
                            self.finalized = true;
                            break;
                        }
                    } else {
                        break;
                    }
                    let rle_result = rle(&buf_current, rle_count, rle_last_char);
                    let mut rle_next = rle_result.data;
                    let rle_next_count = rle_result.counter;
                    let rle_next_char = rle_result.last_byte;

                    let next_data_len = rle_data.len() + rle_next.len();
                    rle_total_count = rle_total_size(next_data_len, rle_next_count, rle_next_char);

                    rle_data.append(&mut rle_next);
                    buf.append(&mut buf_current);
                    rle_count = rle_next_count;
                    rle_last_char = rle_next_char;
                }

                if buf.len() == 0 {
                    break;
                }

                let rle_total = rle_augment(&rle_data, rle_count, rle_last_char);
                let computed_crc = crc32(&buf);
                worker_thread.send_work((computed_crc, rle_total));
            }

            for worker_thread in worker_threads.iter_mut() {
                if worker_thread.pending {
                    worker_thread.flush_work_buffer(&mut self.bit_writer, &mut self.total_crc);
                }
            }
        }

        if self.finalized && !self.encoded {
            self.bit_writer
                .write_bits(&stream_footer(self.total_crc))
                .unwrap();
            self.bit_writer.finalize().unwrap();
            self.encoded = true;
        }

        let res = self.bit_writer.pull(buf.len());
        buf[0..res.len()].copy_from_slice(&res);

        Ok(res.len())
    }
}

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

type Work = (u32, Vec<u8>);
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
                    let (computed_crc, rle_data) = work;
                    send_result
                        .send(generate_block_data(
                            computed_crc,
                            &rle_data,
                            encoding_strategy,
                        ))
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
