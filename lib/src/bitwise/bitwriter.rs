use std::{io::Write, iter::repeat};

use super::Bit;

pub struct BitWriterImpl<'a, T>
where
    T: Write,
{
    pending_bits: Vec<Bit>,
    byte_writer: &'a mut T,
}

impl<'a, T> BitWriterImpl<'a, T>
where
    T: Write,
{
    pub fn from_writer(byte_writer: &'a mut T) -> Self {
        BitWriterImpl {
            pending_bits: vec![],
            byte_writer,
        }
    }
}

pub trait BitWriter {
    fn write_bits(&mut self, bits_to_write: &[Bit]) -> Result<(), ()>;
    fn finalize(&mut self) -> Result<(), ()>;
}

impl<W: BitWriter> BitWriter for &mut W {
    fn write_bits(&mut self, bits_to_write: &[Bit]) -> Result<(), ()> {
        (**self).write_bits(bits_to_write)
    }

    fn finalize(&mut self) -> Result<(), ()> {
        (**self).finalize()
    }
}

impl<'a, T> BitWriter for BitWriterImpl<'a, T>
where
    T: Write,
{
    fn write_bits(&mut self, bits_to_write: &[Bit]) -> Result<(), ()> {
        self.pending_bits.append(&mut bits_to_write.to_vec());
        let mut chunks = self.pending_bits.chunks_exact(8);
        for chunk in &mut chunks {
            let number = convert_to_number(chunk);
            self.byte_writer.write(&[number as u8]).map_err(|_| ())?;
        }
        self.pending_bits = chunks.remainder().to_vec();
        Ok(())
    }

    fn finalize(&mut self) -> Result<(), ()> {
        if self.pending_bits.is_empty() {
            return Ok(());
        }
        let mut trailing_zeros = vec![Bit::Zero; 8 - self.pending_bits.len()];
        self.pending_bits.append(&mut trailing_zeros);
        let byte = convert_to_number(&self.pending_bits);
        self.byte_writer.write(&[byte as u8]).map_err(|_| ())?;

        Ok(())
    }
}

pub fn convert_to_number(input: &[Bit]) -> usize {
    let mut ct = 0;
    for digit in input.iter() {
        ct *= 2;
        match digit {
            Bit::Zero => {}
            Bit::One => ct += 1,
        }
    }
    ct
}

pub fn convert_to_code_pad_to_byte(mut input: u8) -> Vec<Bit> {
    let mut output = Vec::new();
    let mut counter = 0;
    while input > 0 {
        match input & 1 {
            0 => output.push(Bit::Zero),
            _ => output.push(Bit::One),
        }
        input >>= 1;
        counter += 1;
    }
    output.append(
        &mut repeat(Bit::Zero)
            .take(8 - counter as usize)
            .collect::<Vec<_>>(),
    );
    output.reverse();
    output
}

pub fn convert_to_code_pad_to_bytes(input: &[u8]) -> Vec<Bit> {
    input
        .iter()
        .flat_map(|x| convert_to_code_pad_to_byte(*x))
        .collect::<Vec<_>>()
}

pub fn convert_to_code_pad_to_15_bits(mut input: u16) -> Vec<Bit> {
    let mut output = Vec::new();
    let mut counter = 0;
    while input > 0 {
        match input & 1 {
            0 => output.push(Bit::Zero),
            _ => output.push(Bit::One),
        }
        input >>= 1;
        counter += 1;
    }
    output.append(
        &mut repeat(Bit::Zero)
            .take(15 - counter as usize)
            .collect::<Vec<_>>(),
    );
    output.reverse();
    output
}

pub fn convert_to_code_pad_to_n_bits(mut input: usize, n: usize) -> Vec<Bit> {
    let mut output = Vec::new();
    let mut counter = 0;
    while input > 0 {
        match input & 1 {
            0 => output.push(Bit::Zero),
            _ => output.push(Bit::One),
        }
        input >>= 1;
        counter += 1;
    }
    output.append(&mut (counter..n).map(|_| Bit::Zero).collect::<Vec<_>>());
    output.reverse();
    output
}

pub fn increment_symbol(input: Vec<Bit>) -> Vec<Bit> {
    let len = input.len();
    convert_to_code_pad_to_n_bits(convert_to_number(&input) + 1, len)
}
#[cfg(test)]
mod test {
    use super::*;

    fn convert_to_code(mut input: usize) -> Vec<Bit> {
        let mut output = Vec::new();
        while input > 0 {
            match input & 1 {
                0 => output.push(Bit::Zero),
                _ => output.push(Bit::One),
            }
            input >>= 1;
        }

        output.reverse();
        output
    }

    #[test]
    pub fn converts_correctly() {
        assert_eq!(convert_to_number(&convert_to_code(1)), 1);
    }

    #[test]
    pub fn converts_correctly_2() {
        assert_eq!(convert_to_number(&convert_to_code(3)), 3);
    }

    #[test]
    pub fn converts_correctly_more_than_one_byte() {
        assert_eq!(convert_to_number(&convert_to_code(256)), 256);
        assert_eq!(
            convert_to_code(256),
            vec![
                Bit::One,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
            ]
        )
    }

    #[test]
    pub fn increments_more_than_one_byte() {
        assert_eq!(
            increment_symbol(convert_to_code(255)),
            vec![
                Bit::One,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
            ]
        )
    }

    #[test]
    pub fn converts_correctly_3() {
        assert_eq!(
            convert_to_code(convert_to_number(&[Bit::One, Bit::Zero])),
            vec![Bit::One, Bit::Zero]
        );
    }

    #[test]
    pub fn sanity_check() {
        assert_eq!(
            convert_to_code_pad_to_byte(3),
            vec![
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::One,
                Bit::One
            ]
        );
    }

    #[test]
    pub fn increments_padded() {
        assert_eq!(
            increment_symbol(vec![Bit::Zero, Bit::Zero]),
            vec![Bit::Zero, Bit::One]
        );
    }

    #[test]
    pub fn sanity_check_2() {
        assert_eq!(convert_to_number(&[Bit::One, Bit::One, Bit::Zero]), 6);
    }

    #[test]
    pub fn writes_bits() {
        let mut buf = vec![];
        {
            let mut bit_writer = BitWriterImpl::from_writer(&mut buf);
            bit_writer.write_bits(&[Bit::One]).unwrap();
            bit_writer.finalize().unwrap();
        }
        assert_eq!(buf, vec![128]);
    }

    #[test]
    pub fn writes_bits_2() {
        let mut buf = vec![];
        {
            let mut bit_writer = BitWriterImpl::from_writer(&mut buf);
            bit_writer
                .write_bits(&[
                    Bit::One,
                    Bit::One,
                    Bit::One,
                    Bit::One,
                    Bit::One,
                    Bit::One,
                    Bit::One,
                    Bit::One,
                ])
                .unwrap();
        }

        assert_eq!(buf, vec![255]);
    }

    #[test]
    pub fn padding() {
        let mut buf = vec![];

        {
            let mut bit_writer = BitWriterImpl::from_writer(&mut buf);

            bit_writer
                .write_bits(&[
                    Bit::One,
                    Bit::One,
                    Bit::One,
                    Bit::One,
                    Bit::One,
                    Bit::One,
                    Bit::One,
                    Bit::One,
                    Bit::One,
                ])
                .unwrap();
            bit_writer.finalize().unwrap();
        }
        assert_eq!(buf, vec![255, 128]);
    }
}
