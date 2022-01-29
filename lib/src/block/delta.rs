#[derive(Clone, PartialEq, Debug)]
pub enum DeltaEncoded {
    Empty,
    NonEmpty(DeltaEncodedSome),
}
#[derive(Clone, PartialEq, Debug)]

pub struct DeltaEncodedSome {
    pub start_value: u8,
    pub deltas: Vec<DeltaSymbol>,
}
#[derive(Clone, PartialEq, Debug)]

pub enum DeltaSymbol {
    Decrease,
    Increase,
    Stop,
}

pub fn encode_delta(input: Vec<u8>) -> DeltaEncoded {
    let mut delta_codes = Vec::<DeltaSymbol>::new();
    let start_value = input.get(0);
    delta_codes.push(DeltaSymbol::Stop);

    match start_value {
        Some(value) => {
            let mut last_value = value;

            for current_value in input.iter().skip(1) {
                let diff = (*last_value as isize) - (*current_value as isize);
                match diff.cmp(&0) {
                    std::cmp::Ordering::Greater => {
                        for _ in 0..diff.abs() {
                            delta_codes.push(DeltaSymbol::Decrease);
                        }
                    }
                    std::cmp::Ordering::Less => {
                        for _ in 0..diff.abs() {
                            delta_codes.push(DeltaSymbol::Increase);
                        }
                    }

                    std::cmp::Ordering::Equal => (),
                }

                delta_codes.push(DeltaSymbol::Stop);

                last_value = current_value;
            }
            DeltaEncoded::NonEmpty(DeltaEncodedSome {
                start_value: *value,
                deltas: delta_codes,
            })
        }
        None => DeltaEncoded::Empty,
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn encodes() {
        let res = encode_delta(vec![1, 2, 2, 1, 3, 1]);
        assert_eq!(
            res,
            DeltaEncoded::NonEmpty(DeltaEncodedSome {
                start_value: 1,
                deltas: vec![
                    DeltaSymbol::Stop,
                    DeltaSymbol::Increase,
                    DeltaSymbol::Stop,
                    DeltaSymbol::Stop,
                    DeltaSymbol::Decrease,
                    DeltaSymbol::Stop,
                    DeltaSymbol::Increase,
                    DeltaSymbol::Increase,
                    DeltaSymbol::Stop,
                    DeltaSymbol::Decrease,
                    DeltaSymbol::Decrease,
                    DeltaSymbol::Stop
                ]
            })
        )
    }
}
