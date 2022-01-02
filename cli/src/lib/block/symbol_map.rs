use crate::bitwise::{bitreader::BitReader, bitwriter::Bit};

pub(crate) fn get_symbol_table(table: Vec<u8>) -> Vec<Bit> {
    let mut table2 = table.clone();
    table2.sort_unstable();
    let mut out_region = Vec::new();
    let mut out_detail = Vec::new();

    let mut used_symbols_details: [Option<[Bit; 16]>; 16] = [None; 16];
    let mut table2 = table.clone();
    table2.sort_unstable();

    for value in table.iter() {
        let table_number = (value / 16) as usize;
        let entry_in_table = (value % 16) as usize;

        match &mut used_symbols_details[table_number] {
            Some(given) => given[entry_in_table] = Bit::One,
            None => {
                let mut detail_at_position = [Bit::Zero; 16];
                detail_at_position[entry_in_table] = Bit::One;
                used_symbols_details[table_number] = Some(detail_at_position)
            }
        }
    }

    for (_, used) in used_symbols_details.iter().enumerate() {
        match used {
            Some(detail) => {
                out_region.push(Bit::One);
                for a in detail.iter() {
                    out_detail.push(*a);
                }
            }
            None => {
                out_region.push(Bit::Zero);
            }
        }
    }
    out_region.append(&mut out_detail);

    out_region
}

fn get_used_regions(input: &[Bit]) -> Vec<u8> {
    let mut out = vec![];
    for (count, bit) in input.iter().enumerate() {
        if matches!(bit, Bit::One) {
            out.push(count.try_into().unwrap());
        }
    }
    out
}

fn get_used_symbols_from_regions(regions: &[u8], input: Vec<Bit>) -> Vec<u8> {
    regions
        .iter()
        .flat_map(|x| vec![x; 16].into_iter().enumerate())
        .zip(input)
        .flat_map(|((position, region), used)| match used {
            Bit::Zero => vec![],
            Bit::One => vec![region * 16 + u8::try_from(position).unwrap()],
        })
        .collect::<Vec<u8>>()
}

pub(crate) trait GetSymbolTable {
    fn get_symbol_table(&mut self) -> Result<Vec<u8>, ()>;
}

impl<T> GetSymbolTable for T
where
    T: BitReader,
{
    fn get_symbol_table(&mut self) -> Result<Vec<u8>, ()> {
        let index = self.read_bits(16)?;
        let used_regions = get_used_regions(&index);
        let regions = self.read_bits(16 * used_regions.len())?;
        Ok(get_used_symbols_from_regions(&used_regions, regions))
    }
}

#[cfg(test)]

mod test {
    use std::iter::repeat;

    use super::*;
    #[test]
    pub fn one_symbol() {
        let out = &get_symbol_table(vec![0]);
        let mut expected = vec![Bit::One];

        expected.append(&mut zeros(15));
        assert_eq!(&out[0..16], expected);
        assert_eq!(&out[16..32], expected);
        assert_eq!(out.len(), 32);
    }

    #[test]
    pub fn two_symbols_in_same_range() {
        let out = &get_symbol_table(vec![0, 1]);
        let mut expected = vec![Bit::One];

        expected.append(&mut zeros(15));
        assert_eq!(&out[0..16], expected);

        let mut expected = vec![Bit::One, Bit::One];
        expected.append(&mut zeros(14));

        assert_eq!(&out[16..32], expected);
        assert_eq!(out.len(), 32);
    }

    #[test]
    pub fn two_symbols_in_different_ranges() {
        let out = &get_symbol_table(vec![0, 16]);
        let mut expected_overview = vec![Bit::One, Bit::One];

        expected_overview.append(&mut zeros(14));
        assert_eq!(&out[0..16], expected_overview);

        let mut expected_detail = vec![Bit::One];
        expected_detail.append(&mut zeros(15));

        assert_eq!(&out[16..32], expected_detail);
        assert_eq!(&out[32..48], expected_detail);

        assert_eq!(out.len(), 48);
    }

    #[test]
    pub fn two_symbols_in_non_neighboured_ranges() {
        let out = &get_symbol_table(vec![0, 32]);
        let mut expected_overview = vec![Bit::One, Bit::Zero, Bit::One];

        expected_overview.append(&mut zeros(13));
        assert_eq!(&out[0..16], expected_overview);

        let mut expected_detail = vec![Bit::One];
        expected_detail.append(&mut zeros(15));

        assert_eq!(&out[16..32], expected_detail);
        assert_eq!(&out[32..48], expected_detail);

        assert_eq!(out.len(), 48);
    }

    fn zeros(num: usize) -> Vec<Bit> {
        repeat(Bit::Zero).take(num).collect::<Vec<_>>()
    }

    #[test]
    pub fn decodes() {
        let bit_pattern = vec![
            Bit::Zero,
            Bit::One,
            Bit::Zero,
            Bit::One,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::One,
        ];

        assert_eq!(get_used_regions(&bit_pattern), vec![1, 3, 15]);
    }

    #[test]
    pub fn computes_used_symbol() {
        let bit_pattern = vec![
            Bit::Zero,
            Bit::One,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
        ];
        let used_symbols = get_used_symbols_from_regions(&[1], bit_pattern);
        assert_eq!(used_symbols, vec![17]);
    }

    #[test]
    pub fn computes_more_used_symbols() {
        let mut bit_pattern = vec![
            Bit::Zero,
            Bit::One,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
        ];
        bit_pattern.append(&mut bit_pattern.clone());
        let used_symbols = get_used_symbols_from_regions(&[1, 2], bit_pattern);
        assert_eq!(used_symbols, vec![17, 33]);
    }

    #[test]
    pub fn computes_even_more_used_symbols() {
        let mut bit_pattern = vec![
            Bit::Zero,
            Bit::One,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
        ];
        let mut second_bit_pattern = vec![
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::Zero,
            Bit::One,
        ];
        bit_pattern.append(&mut bit_pattern.clone());
        second_bit_pattern.append(&mut bit_pattern);
        let used_symbols = get_used_symbols_from_regions(&[0, 1, 2], second_bit_pattern);
        assert_eq!(used_symbols, vec![15, 17, 33]);
    }
}
