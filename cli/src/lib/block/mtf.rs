use std::collections::{VecDeque};

pub struct MtfData {
    pub encoded: Vec<u8>,
    pub used_symbols: Vec<u8>,
}

pub fn mtf(mtf_input: &[u8]) -> MtfData {
    let mut mtf_result = Vec::<u8>::new();
    let (mut dict, used_symbols) = create_dict(mtf_input);

    for value in mtf_input.iter() {
        let pos = find_pos(*value, &dict);
        mtf_result.push(pos);
        bring_to_front_of_dict(pos, &mut dict);
    }

    MtfData {
        encoded: mtf_result,
        used_symbols,
    }
}

fn create_dict(input: &[u8]) -> (VecDeque<u8>, Vec<u8>) {
    let mut dict = vec![false;255];
    for i in input {
        dict[*i as usize]=true;
    }
    let used_symbols = dict
        .iter()
        .enumerate()
        .filter( |x| *x.1 ).map(|x| x.0 as u8).collect::<Vec<_>>();
    let mut dict_vec = used_symbols.clone();

    dict_vec.sort_unstable();
    (dict_vec.into(), used_symbols)
}

fn bring_to_front_of_dict(position: u8, dict: &mut VecDeque<u8>) {
    let el = dict.remove(usize::from(position)).unwrap();
    dict.push_front(el);
}

fn find_pos(i: u8, dict: &VecDeque<u8>) -> u8 {
    dict.iter().enumerate().find(|(_, x)| **x == i).unwrap().0 as u8
}

pub(crate) fn inverse_mtf(input: &[u8], dictionary: &[u8]) -> Vec<u8> {
    let mut output = vec![];
    let mut dictionary: VecDeque<u8> = dictionary.to_vec().into();

    for i in input {
        let value = dictionary[usize::from(*i)];
        output.push(value);
        let pos = dictionary
            .iter()
            .enumerate()
            .find(|(_, this_value)| **this_value == value)
            .unwrap()
            .0;
        bring_to_front_of_dict(u8::try_from(pos).unwrap(), &mut dictionary);
    }
    output
}

#[cfg(test)]
mod test {
    use std::collections::VecDeque;

    use crate::lib::block::mtf::{bring_to_front_of_dict, inverse_mtf, mtf};


    #[test]
    pub fn brings_to_front_of_dictionary() {
        let mut dict: VecDeque<u8> = vec![1, 2, 3, 4].into();
        bring_to_front_of_dict(3, &mut dict);
        assert_eq!(dict, vec![4, 1, 2, 3]);
    }

    #[test]
    pub fn mtf_easy_sample() {
        let input = b"nnbaaaa";

        let res: Vec<u8> = vec![2, 0, 2, 2, 0, 0, 0];
        assert_eq!(mtf(&input.to_vec()).encoded, res);
    }

    #[test]
    pub fn inverse_mtf_easy_sample() {
        let input = vec![2, 0, 2, 2, 0, 0, 0];

        let res: Vec<u8> = b"nnbaaaa".to_vec();
        assert_eq!(inverse_mtf(&input, b"abn"), res);
    }
}
