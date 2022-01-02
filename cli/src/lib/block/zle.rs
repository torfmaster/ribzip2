use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

#[derive(PartialEq, Clone, Debug, Hash, Eq)]
pub(crate) enum ZleSymbol {
    RunA,
    RunB,
    Number(u8),
}

impl PartialOrd for ZleSymbol {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self {
            ZleSymbol::RunA => match other {
                ZleSymbol::RunA => Some(Ordering::Equal),
                ZleSymbol::RunB => Some(Ordering::Less),
                ZleSymbol::Number(_) => Some(Ordering::Less),
            },
            ZleSymbol::RunB => match other {
                ZleSymbol::RunA => Some(Ordering::Greater),
                ZleSymbol::RunB => Some(Ordering::Equal),
                ZleSymbol::Number(_) => Some(Ordering::Less),
            },
            ZleSymbol::Number(this_number) => match other {
                ZleSymbol::RunA => Some(Ordering::Greater),
                ZleSymbol::RunB => Some(Ordering::Greater),
                ZleSymbol::Number(other_number) => this_number.partial_cmp(other_number),
            },
        }
    }
}

impl Ord for ZleSymbol {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self {
            ZleSymbol::RunA => match other {
                ZleSymbol::RunA => Ordering::Equal,
                ZleSymbol::RunB => Ordering::Less,
                ZleSymbol::Number(_) => Ordering::Less,
            },
            ZleSymbol::RunB => match other {
                ZleSymbol::RunA => Ordering::Greater,
                ZleSymbol::RunB => Ordering::Equal,
                ZleSymbol::Number(_) => Ordering::Less,
            },
            ZleSymbol::Number(this_number) => match other {
                ZleSymbol::RunA => Ordering::Greater,
                ZleSymbol::RunB => Ordering::Greater,
                ZleSymbol::Number(other_number) => this_number.cmp(other_number),
            },
        }
    }
}

fn convert_remainder(input: u8) -> ZleSymbol {
    match input {
        0 => ZleSymbol::RunA,
        _ => ZleSymbol::RunB,
    }
}

fn encode_zero_amount(number_of_zeros: usize) -> Vec<ZleSymbol> {
    let mut out = Vec::<ZleSymbol>::new();
    let mut num = number_of_zeros + 1;

    while num > 0 {
        out.push(convert_remainder((num as u8) & 1));

        num >>= 1;
    }
    out.split_last().map(|x| x.1.to_owned()).unwrap_or_default()
}

pub(crate) fn zle_transform(
    input: Vec<u8>,
    alphabet_size: usize,
) -> (Vec<ZleSymbol>, HashMap<ZleSymbol, usize>) {
    let mut propability_map = HashMap::<ZleSymbol, usize>::new();
    let mut symbol_set_before = HashSet::<u8>::new();

    for sym in input.iter() {
        symbol_set_before.insert(*sym);
    }

    let mut unused_symbols = (0..alphabet_size).map(|x| x as u8).collect::<HashSet<_>>();

    let mut zle_result = Vec::<ZleSymbol>::new();
    let mut zero_count = 0;
    for i in input {
        unused_symbols.remove(&i);

        if i == 0 {
            zero_count += 1;
        } else {
            if zero_count > 0 {
                let zle_encoded = encode_zero_amount(zero_count);
                zle_result.extend(zle_encoded.clone());
                for x in zle_encoded.iter() {
                    match propability_map.get_mut(x) {
                        Some(value) => {
                            *value += 1;
                        }
                        None => {
                            propability_map.insert(x.clone(), 1);
                        }
                    }
                }
            }
            zero_count = 0;
            zle_result.push(ZleSymbol::Number(i));
            match propability_map.get_mut(&ZleSymbol::Number(i)) {
                Some(value) => {
                    *value += 1;
                }
                None => {
                    propability_map.insert(ZleSymbol::Number(i), 1);
                }
            }
        }
    }
    if zero_count > 0 {
        let zle_encoded = encode_zero_amount(zero_count);
        zle_result.extend(zle_encoded.clone());
        for x in zle_encoded.iter() {
            match propability_map.get_mut(x) {
                Some(value) => {
                    *value += 1;
                }
                None => {
                    propability_map.insert(x.clone(), 1);
                }
            }
        }
    }

    for not_encoded in unused_symbols.iter() {
        propability_map.insert(ZleSymbol::Number(*not_encoded), 0);
    }

    (zle_result, propability_map)
}

pub(crate) fn decode_zle(input: &[ZleSymbol]) -> Vec<u8> {
    let mut output = vec![];
    let mut zeros = vec![];
    for element in input {
        match element {
            ZleSymbol::Number(element) => {
                if !zeros.is_empty() {
                    output.append(&mut vec![0u8; decode_zero_amount(&zeros)]);
                    zeros.clear();
                }
                output.push(*element);
            }
            _ => zeros.push(element.clone()),
        }
    }
    if !zeros.is_empty() {
        output.append(&mut vec![0u8; decode_zero_amount(&zeros)]);
        zeros.clear();
    }
    output
}

fn decode_zero_amount(input: &[ZleSymbol]) -> usize {
    let mut complete = vec![ZleSymbol::RunB];
    let mut input = input.to_vec();
    input.reverse();
    complete.append(&mut input);
    let mut number = 0;
    for bit in complete.iter() {
        number <<= 1;
        match bit {
            ZleSymbol::RunA => (),
            ZleSymbol::RunB => number += 1,
            ZleSymbol::Number(_) => (),
        }
    }
    number - 1
}

#[cfg(test)]

mod test {

    use super::*;

    #[test]
    pub fn decodes_zero_amount() {
        let data = vec![
            (1, vec![ZleSymbol::RunA]),
            (2, vec![ZleSymbol::RunB]),
            (3, vec![ZleSymbol::RunA, ZleSymbol::RunA]),
            (4, vec![ZleSymbol::RunB, ZleSymbol::RunA]),
            (5, vec![ZleSymbol::RunA, ZleSymbol::RunB]),
            (6, vec![ZleSymbol::RunB, ZleSymbol::RunB]),
            (7, vec![ZleSymbol::RunA, ZleSymbol::RunA, ZleSymbol::RunA]),
            (8, vec![ZleSymbol::RunB, ZleSymbol::RunA, ZleSymbol::RunA]),
            (9, vec![ZleSymbol::RunA, ZleSymbol::RunB, ZleSymbol::RunA]),
            (10, vec![ZleSymbol::RunB, ZleSymbol::RunB, ZleSymbol::RunA]),
            (11, vec![ZleSymbol::RunA, ZleSymbol::RunA, ZleSymbol::RunB]),
            (12, vec![ZleSymbol::RunB, ZleSymbol::RunA, ZleSymbol::RunB]),
            (13, vec![ZleSymbol::RunA, ZleSymbol::RunB, ZleSymbol::RunB]),
            (14, vec![ZleSymbol::RunB, ZleSymbol::RunB, ZleSymbol::RunB]),
            (
                63,
                vec![
                    ZleSymbol::RunA,
                    ZleSymbol::RunA,
                    ZleSymbol::RunA,
                    ZleSymbol::RunA,
                    ZleSymbol::RunA,
                    ZleSymbol::RunA,
                ],
            ),
        ];
        for (num, encoded) in data.into_iter() {
            let zeroes = decode_zero_amount(&encoded);
            assert_eq!(zeroes, num);
        }
    }

    #[test]
    pub fn encodes_zero_amount() {
        let data = vec![
            (1, vec![ZleSymbol::RunA]),
            (2, vec![ZleSymbol::RunB]),
            (3, vec![ZleSymbol::RunA, ZleSymbol::RunA]),
            (4, vec![ZleSymbol::RunB, ZleSymbol::RunA]),
            (5, vec![ZleSymbol::RunA, ZleSymbol::RunB]),
            (6, vec![ZleSymbol::RunB, ZleSymbol::RunB]),
            (7, vec![ZleSymbol::RunA, ZleSymbol::RunA, ZleSymbol::RunA]),
            (8, vec![ZleSymbol::RunB, ZleSymbol::RunA, ZleSymbol::RunA]),
            (9, vec![ZleSymbol::RunA, ZleSymbol::RunB, ZleSymbol::RunA]),
            (10, vec![ZleSymbol::RunB, ZleSymbol::RunB, ZleSymbol::RunA]),
            (11, vec![ZleSymbol::RunA, ZleSymbol::RunA, ZleSymbol::RunB]),
            (12, vec![ZleSymbol::RunB, ZleSymbol::RunA, ZleSymbol::RunB]),
            (13, vec![ZleSymbol::RunA, ZleSymbol::RunB, ZleSymbol::RunB]),
            (14, vec![ZleSymbol::RunB, ZleSymbol::RunB, ZleSymbol::RunB]),
            (
                63,
                vec![
                    ZleSymbol::RunA,
                    ZleSymbol::RunA,
                    ZleSymbol::RunA,
                    ZleSymbol::RunA,
                    ZleSymbol::RunA,
                    ZleSymbol::RunA,
                ],
            ),
        ];
        for (num, encoded) in data.into_iter() {
            let zeroes = encode_zero_amount(num);
            assert_eq!(zeroes, encoded);
        }
    }

    #[test]
    fn encodes_zeros() {
        let encoded = zle_transform(vec![0, 0, 0], 1).0;
        assert_eq!(encoded, vec![ZleSymbol::RunA, ZleSymbol::RunA]);
    }

    #[test]
    fn encodes_zeros_and_numbers() {
        let encoded = zle_transform(vec![1, 0, 0, 0], 2).0;
        assert_eq!(
            encoded,
            vec![ZleSymbol::Number(1), ZleSymbol::RunA, ZleSymbol::RunA]
        );
    }

    #[test]
    fn decodes_zeros_and_numbers() {
        let encoded = decode_zle(&[ZleSymbol::Number(1), ZleSymbol::RunA, ZleSymbol::RunA]);
        assert_eq!(encoded, vec![1, 0, 0, 0]);
    }

    #[test]
    fn encodes_zeros_and_trailing_numbers() {
        let encoded = zle_transform(vec![1, 0, 0, 0, 2], 3).0;
        assert_eq!(
            encoded,
            vec![
                ZleSymbol::Number(1),
                ZleSymbol::RunA,
                ZleSymbol::RunA,
                ZleSymbol::Number(2)
            ]
        );
    }

    #[test]
    fn decodes_zeros_and_trailing_numbers() {
        let encoded = decode_zle(&[
            ZleSymbol::Number(1),
            ZleSymbol::RunA,
            ZleSymbol::RunA,
            ZleSymbol::Number(2),
        ]);
        assert_eq!(encoded, vec![1, 0, 0, 0, 2]);
    }

    #[test]
    fn encodes_numbers_and_trailing_zeroes() {
        let encoded = zle_transform(vec![1, 0, 0, 0, 2, 0, 0], 3).0;
        assert_eq!(
            encoded,
            vec![
                ZleSymbol::Number(1),
                ZleSymbol::RunA,
                ZleSymbol::RunA,
                ZleSymbol::Number(2),
                ZleSymbol::RunB
            ]
        );
    }
}
