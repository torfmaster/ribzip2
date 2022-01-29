use crate::{
    lib::bitwise::{
        bitreader::BitReader,
        bitwriter::{convert_to_code_pad_to_n_bits, convert_to_number},
    },
    lib::{bitwise::Bit, block::delta::DeltaSymbol},
};

use super::{
    delta::{encode_delta, DeltaEncoded},
    huffman::CanonicalCodeTable,
};

pub(crate) fn encode_code_table<T>(table: CanonicalCodeTable<T>) -> Vec<Bit> {
    let code_lengths = table
        .0
        .iter()
        .map(|entry| entry.code.len() as u8)
        .collect::<Vec<_>>();
    encode_bit_lengths(&code_lengths)
}

fn encode_bit_lengths(code_lengths: &[u8]) -> Vec<Bit> {
    let mut out = Vec::<Bit>::new();

    let delta = encode_delta(code_lengths.to_vec());
    match delta {
        DeltaEncoded::Empty => {}
        DeltaEncoded::NonEmpty(enc) => {
            out.append(&mut convert_to_code_pad_to_n_bits(
                enc.start_value as usize,
                5,
            ));
            out.append(
                &mut enc
                    .deltas
                    .iter()
                    .flat_map(|d| match d {
                        DeltaSymbol::Decrease => {
                            vec![Bit::One, Bit::One]
                        }
                        DeltaSymbol::Increase => {
                            vec![Bit::One, Bit::Zero]
                        }
                        DeltaSymbol::Stop => vec![Bit::Zero],
                    })
                    .collect::<Vec<Bit>>(),
            );
        }
    }
    out
}

pub(crate) trait ReadDelta {
    fn read_delta(&mut self, amount: usize) -> Result<Vec<u8>, ()>;
}

impl<T> ReadDelta for T
where
    T: BitReader,
{
    fn read_delta(&mut self, amount: usize) -> Result<Vec<u8>, ()> {
        let mut out = vec![];
        let mut read = 0;

        let mut start = convert_to_number(&self.read_bits(5)?);
        loop {
            match &self.read_bits(1)?[..] {
                [Bit::One] => match &self.read_bits(1)?[..] {
                    [Bit::Zero] => start += 1,
                    _ => start -= 1,
                },
                _ => {
                    out.push(start as u8);
                    read += 1;
                }
            }
            if read == amount {
                break;
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::lib::bitwise::{
        bitreader::BitReaderImpl,
        bitwriter::{BitWriter, BitWriterImpl},
    };

    use super::*;

    #[test]
    pub fn reads_table() {
        let lengths = vec![1, 2, 3, 4];
        let mut buf = vec![];
        {
            let mut writer = BitWriterImpl::from_writer(&mut buf);
            writer.write_bits(&encode_bit_lengths(&lengths)).unwrap();
            writer.finalize().unwrap();
        }
        let mut cursor = Cursor::new(&buf);
        let mut bit_reader = BitReaderImpl::from_reader(&mut cursor);
        let read = bit_reader.read_delta(4);

        assert_eq!(lengths, read.unwrap());
    }
}
