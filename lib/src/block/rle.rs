pub fn rle(
    input: &[u8],
    init_counter: usize,
    init_last: Option<u8>,
) -> (Vec<u8>, usize, Option<u8>) {
    let mut output = Vec::<u8>::new();
    let mut counter: usize = init_counter;
    let mut last = init_last;
    for current in input.iter() {
        counter += 1;
        if let Some(last_byte) = last {
            if last_byte != *current || counter > 255 {
                if counter >= 5 {
                    for _ in 0..4 {
                        output.push(last_byte);
                    }
                    output.push((counter - 5) as u8);
                } else {
                    for _ in 0..counter - 1 {
                        output.push(last_byte);
                    }
                }
                counter = 1;
            }
        }

        last = Some(*current);
    }

    (output, counter, last)
}

pub fn rle_total_size(input_len: usize, counter: usize, last: Option<u8>) -> usize {
    let mut output_size = input_len;

    if let Some(_last_byte) = last {
        if counter >= 4 {
            for _ in 0..4 {
                output_size += 1;
            }
            output_size += 1;
        } else {
            for _ in 0..counter {
                output_size += 1;
            }
        }
    }

    output_size
}

pub fn rle_augment(input: &[u8], counter: usize, last: Option<u8>) -> Vec<u8> {
    let mut output = Vec::<u8>::from(input);

    if let Some(last_byte) = last {
        if counter >= 4 {
            for _ in 0..4 {
                output.push(last_byte);
            }
            output.push((counter - 4) as u8);
        } else {
            for _ in 0..counter {
                output.push(last_byte);
            }
        }
    }

    output
}

pub(crate) fn inverse_rle(input: &[u8]) -> Vec<u8> {
    let mut output = vec![];
    let mut equal_count = 0;
    let mut previous = None;
    for el in input {
        if let Some(&previous) = previous {
            if equal_count == 3 {
                output.append(&mut vec![previous; usize::from(*el)]);
                equal_count = 0;
            } else if previous == *el {
                equal_count += 1;
                output.push(*el);
            } else {
                output.push(*el);
                equal_count = 0;
            }
        } else {
            output.push(*el);
        }
        previous = Some(el);
    }
    output
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    pub fn keeps_three_byte_sequences() {
        let rle_result = rle(&[1, 1, 1], 0, None);
        assert_eq!(rle_result, (vec![], 3, Some(1)));

        let (rle_data, rle_count, rle_last_char) = rle_result;
        assert_eq!(rle_total_size(rle_data.len(), rle_count, rle_last_char), 3);

        let rle_total = rle_augment(&rle_data, rle_count, rle_last_char);
        assert_eq!(rle_total, vec![1, 1, 1]);
    }

    #[test]
    pub fn adds_overflow_to_sequence() {
        let rle_result = rle(&[1, 1, 1, 1], 0, None);
        assert_eq!(rle_result, (vec![], 4, Some(1)));

        let (rle_data, rle_count, rle_last_char) = rle_result;
        assert_eq!(rle_total_size(rle_data.len(), rle_count, rle_last_char), 5);

        let rle_total = rle_augment(&rle_data, rle_count, rle_last_char);
        assert_eq!(rle_total, vec![1, 1, 1, 1, 0]);
    }

    #[test]
    pub fn larger_numbers() {
        let rle_result = rle(&[1, 1, 1, 1, 1], 0, None);
        assert_eq!(rle_result, (vec![], 5, Some(1)));

        let (rle_data, rle_count, rle_last_char) = rle_result;
        assert_eq!(rle_total_size(rle_data.len(), rle_count, rle_last_char), 5);

        let rle_total = rle_augment(&rle_data, rle_count, rle_last_char);
        assert_eq!(rle_total, vec![1, 1, 1, 1, 1]);
    }

    #[test]
    pub fn mixed_sequences() {
        let rle_result = rle(&[1, 1, 1, 1, 2, 2, 2], 0, None);
        assert_eq!(rle_result, (vec![1, 1, 1, 1, 0], 3, Some(2)));

        let (rle_data, rle_count, rle_last_char) = rle_result;
        assert_eq!(rle_total_size(rle_data.len(), rle_count, rle_last_char), 8);

        let rle_total = rle_augment(&rle_data, rle_count, rle_last_char);
        assert_eq!(rle_total, vec![1, 1, 1, 1, 0, 2, 2, 2]);
    }

    #[test]
    pub fn mixed_sequences_with_length() {
        let rle_result = rle(&[1, 1, 1, 1, 1, 2, 2, 2, 2, 2], 0, None);
        assert_eq!(rle_result, (vec![1, 1, 1, 1, 1], 5, Some(2)));

        let (rle_data, rle_count, rle_last_char) = rle_result;
        assert_eq!(rle_total_size(rle_data.len(), rle_count, rle_last_char), 10);

        let rle_total = rle_augment(&rle_data, rle_count, rle_last_char);
        assert_eq!(rle_total, vec![1, 1, 1, 1, 1, 2, 2, 2, 2, 1]);
    }

    #[test]
    pub fn mixed_sequences_with_and_without_length() {
        let rle_result = rle(&[1, 1, 1, 2, 2, 2, 2, 2], 0, None);
        assert_eq!(rle_result, (vec![1, 1, 1], 5, Some(2)));

        let (rle_data, rle_count, rle_last_char) = rle_result;
        assert_eq!(rle_total_size(rle_data.len(), rle_count, rle_last_char), 8);

        let rle_total = rle_augment(&rle_data, rle_count, rle_last_char);
        assert_eq!(rle_total, vec![1, 1, 1, 2, 2, 2, 2, 1]);
    }

    #[test]
    pub fn mixed_sequences_with_and_without_length_2() {
        let rle_result = rle(&[1, 1, 1, 1, 2, 2, 2], 0, None);
        assert_eq!(rle_result, (vec![1, 1, 1, 1, 0], 3, Some(2)));

        let (rle_data, rle_count, rle_last_char) = rle_result;
        assert_eq!(rle_total_size(rle_data.len(), rle_count, rle_last_char), 8);

        let rle_total = rle_augment(&rle_data, rle_count, rle_last_char);
        assert_eq!(rle_total, vec![1, 1, 1, 1, 0, 2, 2, 2]);
    }

    #[test]
    pub fn even_longer_sequences() {
        let rle_result = rle(&[1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3], 0, None);
        assert_eq!(rle_result, (vec![1, 1, 1, 1, 2, 2, 2, 2, 2, 2], 3, Some(3)));

        let (rle_data, rle_count, rle_last_char) = rle_result;
        assert_eq!(rle_total_size(rle_data.len(), rle_count, rle_last_char), 13);

        let rle_total = rle_augment(&rle_data, rle_count, rle_last_char);
        assert_eq!(rle_total, vec![1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3]);
    }

    #[test]
    pub fn max_block_length() {
        let rle_result = rle(
            &std::iter::repeat(3).take(255).collect::<Vec<u8>>(),
            0,
            None,
        );
        assert_eq!(rle_result, (vec![], 255, Some(3)));

        let (rle_data, rle_count, rle_last_char) = rle_result;
        assert_eq!(rle_total_size(rle_data.len(), rle_count, rle_last_char), 5);

        let rle_total = rle_augment(&rle_data, rle_count, rle_last_char);
        assert_eq!(rle_total, vec![3, 3, 3, 3, 251]);
    }

    #[test]
    pub fn more_than_max_block_length() {
        let rle_result = rle(
            &std::iter::repeat(3).take(256).collect::<Vec<u8>>(),
            0,
            None,
        );
        assert_eq!(rle_result, (vec![3, 3, 3, 3, 251], 1, Some(3)));

        let (rle_data, rle_count, rle_last_char) = rle_result;
        assert_eq!(rle_total_size(rle_data.len(), rle_count, rle_last_char), 6);

        let rle_total = rle_augment(&rle_data, rle_count, rle_last_char);
        assert_eq!(rle_total, vec![3, 3, 3, 3, 251, 3]);
    }
    #[test]
    pub fn twice_max_block_length() {
        let rle_result = rle(
            &std::iter::repeat(3).take(510).collect::<Vec<u8>>(),
            0,
            None,
        );
        assert_eq!(rle_result, (vec![3, 3, 3, 3, 251], 255, Some(3)));

        let (rle_data, rle_count, rle_last_char) = rle_result;
        assert_eq!(rle_total_size(rle_data.len(), rle_count, rle_last_char), 10);

        let rle_total = rle_augment(&rle_data, rle_count, rle_last_char);
        assert_eq!(rle_total, vec![3, 3, 3, 3, 251, 3, 3, 3, 3, 251]);
    }

    #[test]
    pub fn inverse_rle_works() {
        assert_eq!(
            inverse_rle(&[1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3]),
            vec![1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3]
        );
    }

    #[test]
    pub fn inverse_mixed_sequences_with_and_without_length_2() {
        assert_eq!(
            inverse_rle(&[1, 1, 1, 1, 0, 2, 2, 2]),
            vec![1, 1, 1, 1, 2, 2, 2]
        );
    }
}
