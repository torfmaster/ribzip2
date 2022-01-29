use crate::lib::bitwise::{bitreader::BitReader, Bit};

use super::mtf::mtf;

pub(crate) fn create_selectors(selectors: &[u8]) -> (Vec<Bit>, usize) {
    let mut out = vec![];
    let selectors_mtf = mtf(&selectors);

    for selector in selectors_mtf.encoded {
        for _ in 0..selector as usize {
            out.push(Bit::One);
        }
        out.push(Bit::Zero);
    }
    let l = selectors.len();
    (out, l)
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

    use crate::lib::bitwise::bitreader::InMemoryBitReader;

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
