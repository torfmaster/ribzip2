use crate::lib::bitwise::bitwriter::increment_symbol;
use crate::lib::bitwise::Bit;

use crate::lib::block::symbol_statistics::IntoFrequencyTable;
use crate::lib::block::zle::ZleSymbol;
use std::{fmt::Debug, iter::repeat};

use super::package_merge::compute_lis;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub(crate) enum HuffmanSymbol<T> {
    NormalSymbol(T),
    EoB,
}

#[derive(PartialEq, Debug, Clone, Eq)]
pub(crate) struct CodeTableEntry<T> {
    pub code: usize,
    pub symbol: T,
}

#[derive(PartialEq, Debug, Clone, Eq)]
pub(crate) struct CanonicalCodeTableEntry<T> {
    pub code: Vec<Bit>,
    pub symbol: T,
}

impl<T> PartialOrd for CodeTableEntry<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let CodeTableEntry { symbol, code } = self;
        (code, symbol).partial_cmp(&(&other.code, &other.symbol))
    }
}

impl<T> Ord for CodeTableEntry<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let CodeTableEntry { symbol, code } = self;
        (code, symbol).cmp(&(&other.code, &other.symbol))
    }
}

#[derive(PartialEq, Debug, Clone)]
pub(crate) struct CodeTable<T>(pub(crate) Vec<CodeTableEntry<T>>);

#[derive(Eq, PartialEq, Debug, Clone)]
pub(crate) struct CanonicalCodeTable<T>(pub(crate) Vec<CanonicalCodeTableEntry<T>>);

/// Computes a huffman code table using greedy construction of the codes.
/// The code tables are then limited to 20 bit length using the package merge algorithm.
/// This restriction is due to the original implementation of bzip2 and hence cannot be dropped.
pub(crate) fn compute_huffman(
    frequency_table: IntoFrequencyTable,
) -> CodeTable<HuffmanSymbol<ZleSymbol>> {
    let mut frequency_table = frequency_table
        .iterate()
        .map(|(symbol, frequency)| FrequencyTableEntry {
            frequency,
            symbol: HuffmanSymbol::NormalSymbol(symbol),
        })
        .collect::<Vec<_>>();

    frequency_table.push(FrequencyTableEntry {
        frequency: 0,
        symbol: HuffmanSymbol::EoB,
    });

    frequency_table.sort_by(|x, y| x.frequency.cmp(&y.frequency));

    // Step 1 break ties between frequencies of symbols
    let mut last: Option<FrequencyTableEntry<_>> = None;
    for entry in frequency_table.iter_mut() {
        if let Some(last) = last {
            if last.frequency >= entry.frequency {
                // break ties
                entry.frequency += last.frequency - entry.frequency + 1;
            }
        }

        last = Some(entry.clone());
    }

    let weights = frequency_table
        .iter()
        .map(|x| x.frequency)
        .collect::<Vec<_>>();
    let code_lengths = compute_lis(&weights, 17);

    let code_length_iter = code_lengths
        .iter()
        .zip(frequency_table.iter())
        .map(|(code_length, table_entry)| CodeTableEntry {
            code: *code_length,
            symbol: table_entry.symbol.clone(),
        })
        .collect::<Vec<_>>();
    CodeTable(code_length_iter)
}

impl<T> CodeTable<T>
where
    T: Ord + Clone + std::fmt::Debug,
{
    pub(crate) fn canonicalize(mut self) -> CanonicalCodeTable<T> {
        let mut canonical_code_table_entries = vec![];

        self.0.sort();
        let mut iter = self.0.iter();
        let code_table_entry = iter.next().unwrap();
        let mut last = repeat(Bit::Zero)
            .take(code_table_entry.code)
            .collect::<Vec<_>>();

        canonical_code_table_entries.push(CanonicalCodeTableEntry {
            code: last.clone(),
            symbol: code_table_entry.symbol.clone(),
        });

        for entry in iter {
            let mut new_code_unpadded = increment_symbol(last);

            // pad zeros to length
            let pad = entry.code as isize - new_code_unpadded.len() as isize;
            for _ in 0..pad {
                new_code_unpadded.push(Bit::Zero);
            }

            canonical_code_table_entries.push(CanonicalCodeTableEntry {
                code: new_code_unpadded.clone(),
                symbol: entry.symbol.clone(),
            });

            last = new_code_unpadded;
        }
        // Sort resulting new codes by symbol
        canonical_code_table_entries.sort_by(|x, y| x.symbol.cmp(&y.symbol));
        CanonicalCodeTable(canonical_code_table_entries)
    }

    pub(crate) fn from_weights(code_lengths: &[u8]) -> CodeTable<HuffmanSymbol<ZleSymbol>> {
        let mut symbols = vec![
            HuffmanSymbol::NormalSymbol(ZleSymbol::RunA),
            HuffmanSymbol::NormalSymbol(ZleSymbol::RunB),
        ];
        let mut numbers = (0..code_lengths.len() - 3)
            .map(|x| HuffmanSymbol::NormalSymbol(ZleSymbol::Number(u8::try_from(x).unwrap() + 1u8)))
            .collect::<Vec<_>>();
        symbols.append(&mut numbers);
        symbols.push(HuffmanSymbol::EoB);
        CodeTable(
            symbols
                .into_iter()
                .zip(code_lengths.iter())
                .map(|(symbol, length)| CodeTableEntry {
                    code: usize::from(*length),
                    symbol,
                })
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FrequencyTableEntry<T> {
    pub frequency: usize,
    pub(crate) symbol: T,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn canonicalizes_and_sorts_alphabetically() {
        let table = CodeTable(vec![
            CodeTableEntry {
                code: vec![Bit::One, Bit::One].len(),
                symbol: 'a',
            },
            CodeTableEntry {
                code: vec![Bit::Zero].len(),
                symbol: 'b',
            },
            CodeTableEntry {
                code: vec![Bit::One, Bit::Zero, Bit::One].len(),
                symbol: 'c',
            },
            CodeTableEntry {
                code: vec![Bit::One, Bit::Zero, Bit::Zero].len(),
                symbol: 'd',
            },
        ]);
        let canonical = table.canonicalize();
        assert_eq!(
            CanonicalCodeTable(vec![
                CanonicalCodeTableEntry {
                    code: vec![Bit::One, Bit::Zero],
                    symbol: 'a',
                },
                CanonicalCodeTableEntry {
                    code: vec![Bit::Zero],
                    symbol: 'b',
                },
                CanonicalCodeTableEntry {
                    code: vec![Bit::One, Bit::One, Bit::Zero],
                    symbol: 'c',
                },
                CanonicalCodeTableEntry {
                    code: vec![Bit::One, Bit::One, Bit::One],
                    symbol: 'd',
                },
            ]),
            canonical
        );
    }
}
