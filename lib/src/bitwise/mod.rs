pub mod bitreader;
pub mod bitwriter;

#[derive(Clone, PartialEq, Eq, Debug, PartialOrd, Copy)]
pub enum Bit {
    Zero,
    One,
}
