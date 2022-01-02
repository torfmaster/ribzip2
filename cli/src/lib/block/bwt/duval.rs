fn duval(input: &[u8]) -> usize {
    let mut final_start=0;
    let n = input.len();
    let mut i = 0;

    while i < n {
        let mut j = i + 1;
        let mut k = i;
        while j < n && input[k] <= input[j] {
            if input[k] < input[j] {
                k = i;
            }
            else {
                k+=1;
            }
            j+=1;
        }
        while i <= k {
            final_start= i;

            i += j - k;
        }
    }
    return final_start;
}

/// Compute lexicographically minimal rotation using the duval algorithm.
/// Returns the rotation and the offset.
pub fn rotate_duval(input: & [u8]) -> (Vec<u8>, usize) {
    let offset = duval(input);
    let mut buf = vec![];
    let (head, tail) = input.split_at(offset);
    buf.append(&mut tail.to_vec());
    buf.append(&mut head.to_vec());
    (buf, offset)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn rotates() {
        let rotated = rotate_duval(b"abacabab");
        assert_eq!(rotated.0, b"ababacab")
    }

    #[test]
    pub fn finds_final_lyndon_word() {
        let rotated = rotate_duval(b"bananaa");
        assert_eq!(rotated.1,6)
    }
}
