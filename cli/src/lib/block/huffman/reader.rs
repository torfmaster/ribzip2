use crate::lib::bitwise::{bitreader::BitReader, Bit};

use crate::lib::block::zle::ZleSymbol;

use super::{CanonicalCodeTable, HuffmanSymbol};

pub(crate) trait ReadSymbols {
    fn read_symbols(
        &mut self,
        tree: &CanonicalCodeTable<HuffmanSymbol<ZleSymbol>>,
        max_number: usize,
    ) -> Result<Vec<ZleSymbol>, ()>;
}

impl<T> ReadSymbols for T
where
    T: BitReader,
{
    fn read_symbols(
        &mut self,
        table: &CanonicalCodeTable<HuffmanSymbol<ZleSymbol>>,
        max_number: usize,
    ) -> Result<Vec<ZleSymbol>, ()> {
        let mut all_symbols = vec![];
        let mut current_symbol = vec![];
        let mut symbols_read = 0;

        loop {
            let mut read = self.read_bits(1)?;
            current_symbol.append(&mut read);
            if let Some(symbol) = read_from_table(table, &current_symbol) {
                match symbol {
                    HuffmanSymbol::NormalSymbol(symbol) => {
                        all_symbols.push(symbol);
                        current_symbol.clear();
                        symbols_read += 1;
                    }
                    HuffmanSymbol::EoB => break,
                }
            }
            if symbols_read == max_number {
                break;
            }
        }
        Ok(all_symbols)
    }
}

fn read_from_table(
    table: &CanonicalCodeTable<HuffmanSymbol<ZleSymbol>>,
    symbols: &[Bit],
) -> Option<HuffmanSymbol<ZleSymbol>> {
    for entry in table.0.iter() {
        if symbols == entry.code {
            return Some(entry.symbol.clone());
        }
    }
    None
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::lib::bitwise::{
        bitreader::BitReaderImpl,
        bitwriter::{BitWriter, BitWriterImpl},
    };

    use crate::lib::block::huffman::CanonicalCodeTableEntry;

    use super::*;
    #[test]
    pub fn reads_symbols() {
        let mut buf = vec![];
        let table = CanonicalCodeTable(vec![
            CanonicalCodeTableEntry {
                code: vec![Bit::Zero],
                symbol: HuffmanSymbol::NormalSymbol(ZleSymbol::RunA),
            },
            CanonicalCodeTableEntry {
                code: vec![Bit::One, Bit::Zero],
                symbol: HuffmanSymbol::NormalSymbol(ZleSymbol::RunB),
            },
            CanonicalCodeTableEntry {
                code: vec![Bit::One, Bit::One, Bit::One, Bit::Zero],
                symbol: HuffmanSymbol::NormalSymbol(ZleSymbol::Number(0)),
            },
            CanonicalCodeTableEntry {
                code: vec![Bit::One, Bit::One, Bit::Zero],
                symbol: HuffmanSymbol::NormalSymbol(ZleSymbol::Number(1)),
            },
            CanonicalCodeTableEntry {
                code: vec![Bit::One, Bit::One, Bit::One, Bit::One],
                symbol: HuffmanSymbol::EoB,
            },
        ]);
        let stream = vec![
            Bit::Zero,
            Bit::One,
            Bit::Zero,
            Bit::One,
            Bit::One,
            Bit::One,
            Bit::Zero,
            Bit::One,
            Bit::One,
            Bit::Zero,
            Bit::One,
            Bit::One,
            Bit::One,
            Bit::One,
        ];
        {
            let mut bit_writer = BitWriterImpl::from_writer(&mut buf);
            bit_writer.write_bits(&stream).unwrap();
            bit_writer.finalize().unwrap();
        }
        let bytes = buf;

        let mut cursor = Cursor::new(bytes);

        let mut bit_reader = BitReaderImpl::from_reader(&mut cursor);

        let code = bit_reader.read_symbols(&table, 5).unwrap();

        let expected_code = vec![
            ZleSymbol::RunA,
            ZleSymbol::RunB,
            ZleSymbol::Number(0),
            ZleSymbol::Number(1),
        ];
        assert_eq!(code, expected_code);
    }
}
