use std::io::Read;

use crate::bitwise::bitwriter::convert_to_number;

use super::Bit;

pub struct BitReaderImpl<'a, T: Read> {
    byte_reader: &'a mut T,
    current_byte_cursor: u8,
    current_byte: Option<u8>,
}

pub trait BitReader {
    fn read_bits(&mut self, num: usize) -> Result<Vec<Bit>, ()>;
    fn read_bytes(&mut self, number: usize) -> Result<Vec<u8>, ()> {
        let mut out = vec![];
        for _ in 0..number {
            out.push(convert_to_number(&self.read_bits(8)?).try_into().unwrap());
        }
        Ok(out)
    }
}

impl<T> BitReader for &mut T
where
    T: BitReader,
{
    fn read_bits(&mut self, num: usize) -> Result<Vec<Bit>, ()> {
        (**self).read_bits(num)
    }
}

impl<'a, T: Read> BitReaderImpl<'a, T> {
    pub fn from_reader(reader: &'a mut T) -> Self {
        BitReaderImpl {
            byte_reader: reader,
            current_byte_cursor: 0u8,
            current_byte: None,
        }
    }
}

impl<'a, T: Read> BitReader for BitReaderImpl<'a, T> {
    fn read_bits(&mut self, mut num: usize) -> Result<Vec<Bit>, ()> {
        let mut out = vec![];
        let mut buf = [0u8; 1];

        while num > 0 {
            match self.current_byte {
                Some(current_byte) => {
                    if self.current_byte_cursor >= 8 {
                        self.byte_reader.read_exact(&mut buf).map_err(|_| ())?;
                        self.current_byte = Some(buf[0]);
                        self.current_byte_cursor = 0;
                    } else {
                        let offset = 8 - self.current_byte_cursor - 1;
                        let current_bit = (current_byte >> offset) & 1;
                        out.push(if current_bit == 0 {
                            Bit::Zero
                        } else {
                            Bit::One
                        });

                        self.current_byte_cursor += 1;
                        num -= 1;
                    }
                }
                None => {
                    self.byte_reader.read_exact(&mut buf).map_err(|_| ())?;
                    self.current_byte = Some(buf[0]);
                    self.current_byte_cursor = 0;
                }
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
pub struct InMemoryBitReader {
    bits: Vec<Bit>,
    cursor: usize,
}

#[cfg(test)]
impl InMemoryBitReader {
    pub fn from_bits(bits: &[Bit]) -> Self {
        InMemoryBitReader {
            bits: bits.to_vec(),
            cursor: 0usize,
        }
    }
}

#[cfg(test)]
impl BitReader for InMemoryBitReader {
    fn read_bits(&mut self, num: usize) -> Result<Vec<Bit>, ()> {
        if num + self.cursor > num + self.bits.len() {
            Err(())
        } else {
            let out = self.bits[self.cursor..self.cursor + num].to_vec();
            self.cursor += num;

            Ok(out)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    pub fn reads_0() {
        let vec = vec![0u8, 1, 2, 3];
        let mut cursor = Cursor::new(&vec);
        let mut reader = BitReaderImpl::from_reader(&mut cursor);
        let bits = reader.read_bits(8);
        assert_eq!(
            bits,
            Ok(vec![
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero
            ])
        );
    }

    #[test]
    pub fn reads_8() {
        let vec = vec![8u8, 1, 2, 3];
        let mut cursor = Cursor::new(&vec);
        let mut reader = BitReaderImpl::from_reader(&mut cursor);
        let bits = reader.read_bits(8);
        assert_eq!(
            bits,
            Ok(vec![
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::One,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero
            ])
        );
    }

    #[test]
    pub fn reads_two_times_8() {
        let vec = vec![8u8, 8u8, 2, 3];
        let mut cursor = Cursor::new(&vec);
        let mut reader = BitReaderImpl::from_reader(&mut cursor);
        let _ = reader.read_bits(8);
        let bits = reader.read_bits(8);

        assert_eq!(
            bits,
            Ok(vec![
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::One,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero
            ])
        );
    }

    #[test]
    pub fn reads_9_bits() {
        let vec = vec![0u8, 255u8, 2, 3];
        let mut cursor = Cursor::new(&vec);
        let mut reader = BitReaderImpl::from_reader(&mut cursor);
        let bits_1 = reader.read_bits(9);
        let bits_2 = reader.read_bits(1);

        assert_eq!(
            bits_1,
            Ok(vec![
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::One,
            ])
        );
        assert_eq!(bits_2, Ok(vec![Bit::One]));
    }

    #[test]
    pub fn reads_bytes_then_bits() {
        let vec = vec![42u8, 42u8, 2, 3];
        let mut cursor = Cursor::new(&vec);
        let mut reader = BitReaderImpl::from_reader(&mut cursor);
        let _ = reader.read_bits(8);
        let bytes = reader.read_bytes(1).unwrap();
        let bits = reader.read_bits(8).unwrap();
        assert_eq!(bytes, vec![42]);
        assert_eq!(
            bits,
            vec![
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::Zero,
                Bit::One,
                Bit::Zero,
            ]
        );
    }
}
