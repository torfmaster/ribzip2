use super::{sais::{build_suffix_array}, duval::rotate_duval};

fn bwt_private(string: &[u8]) -> (Vec<u8>, usize) {
    let (rotated, shift) = rotate_duval(&string);

    let entries = build_suffix_array(&rotated);
    let len = string.len();
    let bwt = entries
        .iter()
        .filter(|x| x.index < len)
        .map(|x| {
            let index = if x.index > 0 { x.index - 1 } else { len - 1 };
            rotated[index]
        })
        .collect::<Vec<_>>();
    let orig_ptr = entries
        .iter()
        .filter(|x| x.index < len)
        .enumerate()
        .find(|(_, x)| x.index == (len-shift)%len)
        .unwrap()
        .0;
    (bwt, orig_ptr)
}

/// Computes the Burrows-Wheeler-Transform of the input data without a sentinel value.
/// It uses the duval algorithm to provide a lexicographically minimal rotation of the input string
/// and passes this to the SAIS algorithm. The rotation makes sure that the BWT is computed
/// correctly because the rotation is lexicographically minimal.
pub fn bwt(input: Vec<u8>) -> BwtData {
    let res = bwt_private(&input);

    BwtData {
        data: res.0,
        end_of_string: res.1 as u32,
    }
}

#[derive(Debug, Clone)]
pub struct BwtData {
    pub data: Vec<u8>,
    pub end_of_string: u32,
}

#[cfg(test)]
mod test {
    use super::bwt;

    #[test]
    pub fn banana() {
        let bwt_result = bwt(b"banana".to_vec());
        assert_eq!(bwt_result.data, b"nnbaaa".to_vec());
    }

    #[test]
    pub fn bananaa() {
        let bwt_result = bwt(b"bananaa".to_vec());
        assert_eq!(bwt_result.data, b"nanbaaa".to_vec());
    }

    #[test]
    pub fn banana2() {
        let bwt_result = bwt(b"banana".to_vec());
        assert_eq!(bwt_result.data, b"nnbaaa".to_vec());
        assert_eq!(bwt_result.end_of_string, 3);
    }

    #[test]
    pub fn longer_text() {
        let bwt_result = bwt(b"If Peter Piper picked a peck of pickled peppers, where's the peck of pickled peppers Peter Piper picked?????".to_vec());
        assert_eq!(24, bwt_result.end_of_string);
        assert_eq!(b"fsrrdkkeaddrrffs,esd?????     eeiiiieeeehrppkllkppttpphppPPIootwppppPPcccccckk      iipp    eeeeeeeeer'ree  ".to_vec(), bwt_result.data);
    }

    #[test]
    pub fn banana3() {
        let bwt_result = bwt(b"bananaaar".to_vec());
        assert_eq!(bwt_result.data, b"nanbaraaa".to_vec());
        assert_eq!(bwt_result.end_of_string, 5);
    }
}
