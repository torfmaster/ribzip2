use self::kmeansclustering::KMeansProblem;

use super::zle::ZleSymbol;
mod kmeansclustering;

#[derive(Clone, Copy)]
#[non_exhaustive]
/// Strategy for using Huffman Tables
///
/// * [EncodingStrategy::Single] - every block of 900k uses a single huffman code table
/// * [EncodingStrategy::BlockWise] - usage of code tables in 50 byte chunks is optimized using Lloyd's algorithm with given parameters
pub enum EncodingStrategy {
    BlockWise {
        num_clusters: usize,
        num_iterations: usize,
    },
    Single,
}

pub(crate) struct SinglePropabilityMap {
    frequencies: Vec<usize>,
    symbol_count: usize,
}

impl SinglePropabilityMap {
    pub(crate) fn create(size: usize) -> Self {
        SinglePropabilityMap {
            frequencies: vec![0; size + 1],
            symbol_count: 0,
        }
    }
}

impl SymbolReporter for SinglePropabilityMap {
    fn report_symbol(&mut self, symbol: &ZleSymbol) {
        self.symbol_count += 1;
        match symbol {
            ZleSymbol::RunA => {
                self.frequencies[0] += 1;
            }
            ZleSymbol::RunB => {
                self.frequencies[1] += 1;
            }
            ZleSymbol::Number(i) => {
                self.frequencies[*i as usize + 1] += 1;
            }
        }
    }

    fn finalize(&mut self) -> ReportedSymbols {
        let table = IntoFrequencyTable {
            frequencies: self.frequencies.clone(),
        };
        ReportedSymbols {
            reported_frequencies: vec![table.clone(), table],
            selectors: vec![0; (self.symbol_count as f32 / 50.0).ceil() as usize],
        }
    }
}

pub(crate) struct BlockWisePropabilityMap {
    current_frequencies: Vec<u8>,
    pub(crate) maps: Vec<Vec<u8>>,
    pub(crate) size: usize,
    counter: usize,
    num_iterations: usize,
    num_clusters: usize,
}

impl BlockWisePropabilityMap {
    pub(crate) fn create(size: usize, num_clusters: usize, num_iterations: usize) -> Self {
        Self {
            current_frequencies: vec![0; size + 1],
            maps: vec![],
            counter: 0,
            size,
            num_clusters,
            num_iterations,
        }
    }
}

impl SymbolReporter for BlockWisePropabilityMap {
    fn report_symbol(&mut self, symbol: &ZleSymbol) {
        match symbol {
            ZleSymbol::RunA => {
                self.current_frequencies[0] += 1;
            }
            ZleSymbol::RunB => {
                self.current_frequencies[1] += 1;
            }
            ZleSymbol::Number(i) => {
                self.current_frequencies[*i as usize + 1] += 1;
            }
        }
        self.counter += 1;
        if self.counter >= 50 {
            self.maps.push(std::mem::replace(
                &mut self.current_frequencies,
                vec![0; self.size + 1],
            ));
            self.counter = 0;
        }
    }

    fn finalize(&mut self) -> ReportedSymbols {
        self.maps.push(std::mem::replace(
            &mut self.current_frequencies,
            vec![0; self.size + 1],
        ));

        let p = KMeansProblem {
            dimension: self.size + 1,
            data: &self.maps,
            num_iterations: self.num_iterations,
            num_clusters: self.num_clusters,
        };
        let tables = p.solve();

        ReportedSymbols {
            reported_frequencies: tables
                .means
                .iter()
                .map(|x| IntoFrequencyTable {
                    frequencies: x.iter().map(|x| *x as usize).collect::<Vec<_>>(),
                })
                .collect::<Vec<_>>(),
            selectors: tables.assignments,
        }
    }
}

pub(crate) trait SymbolReporter {
    fn report_symbol(&mut self, symbol: &ZleSymbol);
    fn finalize(&mut self) -> ReportedSymbols;
}

pub(crate) struct ReportedSymbols {
    pub(crate) reported_frequencies: Vec<IntoFrequencyTable>,
    pub(crate) selectors: Vec<u8>,
}

#[derive(Clone)]
pub(crate) struct IntoFrequencyTable {
    pub(crate) frequencies: Vec<usize>,
}

impl IntoFrequencyTable {
    pub(crate) fn iterate(self) -> impl Iterator<Item = (ZleSymbol, usize)> {
        self.frequencies
            .into_iter()
            .enumerate()
            .map(|(symbol, frequency)| match symbol {
                0 => (ZleSymbol::RunA, frequency),
                1 => (ZleSymbol::RunB, frequency),
                x => (ZleSymbol::Number((x - 1) as u8), frequency),
            })
    }
}
