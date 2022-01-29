pub(crate) fn inverse_bwt(data: &[u8], orig_ptr: usize) -> Vec<u8> {
    let mut out = vec![];
    let mut pairs = data.iter().enumerate().collect::<Vec<_>>();
    pairs.sort_by(|x, y| x.1.cmp(y.1));
    let mut start = pairs.iter().find(|(x, _)| *x == orig_ptr).unwrap().0;

    for _ in 0..data.len() {
        let pair = pairs[start];
        out.push(*pair.1);
        start = pair.0;
    }
    out
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    pub fn test() {
        let res = inverse_bwt(b"nnbaaa", 3);
        assert_eq!(res, b"banana".to_vec());
    }

    #[test]
    pub fn longer_test() {
        let transformed = b"fsrrdkkeaddrrffs,esd?????     eeiiiieeeehrppkllkppttpphppPPIootwppppPPcccccckk      iipp    eeeeeeeeer'ree  ";
        let orig_ptr = 24;
        let original = b"If Peter Piper picked a peck of pickled peppers, where's the peck of pickled peppers Peter Piper picked?????".to_vec();

        let res = inverse_bwt(transformed, orig_ptr);
        assert_eq!(res, original);
    }
}
