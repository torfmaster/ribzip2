use std::iter::repeat;

use crate::bitwise::{bitreader::BitReader, bitwriter::Bit};


// FIXME: only supports one (duplicated huffman table)
pub(crate) fn create_selectors(len: usize) -> (Vec<Bit>, usize) {
    let num_selectors = len / 50;
    let remainder = len % 50;

    let num_selectors = num_selectors + if remainder > 0 { 1 } else { 0 };

    (
        repeat(Bit::Zero).take(num_selectors).collect::<Vec<_>>(),
        num_selectors,
    )
}

pub(crate) trait ReadUnary {
    fn read_unary(&mut self, amount: usize) -> Result<Vec<u8>, ()>;
}

impl<T> ReadUnary for T
where
    T: BitReader,
{
    fn read_unary(&mut self, amount: usize) -> Result<Vec<u8>, ()> {
        let mut output = vec![];
        let mut current_symbol = 0u8;
        let mut symbol_count = 0;
        loop {
            match &self.read_bits(1)?[..] {
                [Bit::One] => current_symbol += 1,
                [Bit::Zero] => {
                    symbol_count += 1;
                    output.push(current_symbol);
                    current_symbol = 0;
                }
                _ => return Err(()),
            }
            if symbol_count >= amount {
                break;
            }
        }
        Ok(output)
    }
}

#[cfg(test)]
mod test {


    use crate::bitwise::bitreader::InMemoryBitReader;

    use super::*;

    #[test]
    pub fn decodes_one_unary() {
        let encoded = vec![
            Bit::One,
            Bit::One,
            Bit::Zero,
            Bit::One,
            Bit::One,
            Bit::Zero,
            Bit::One,
            Bit::Zero,
        ];

        let mut bit_reader = InMemoryBitReader::from_bits(&encoded);
        assert_eq!(bit_reader.read_unary(1).unwrap(), vec![2]);
    }

    #[test]
    pub fn decodes_zero_unary() {
        let encoded = vec![
            Bit::One,
            Bit::Zero,
            Bit::Zero,
            Bit::One,
            Bit::One,
            Bit::Zero,
            Bit::One,
            Bit::Zero,
        ];
        let mut bit_reader = InMemoryBitReader::from_bits(&encoded);

        assert_eq!(bit_reader.read_unary(4).unwrap(), vec![1, 0, 2, 1]);
    }
}
