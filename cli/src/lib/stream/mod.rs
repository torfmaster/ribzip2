use crate::bitwise::bitwriter::{convert_to_code_pad_to_byte, Bit};

use crate::lib::block::block_data::crc_as_bytes;

pub fn stream_footer(crc: u32) -> Vec<Bit> {
    let mut out = vec![];

    out.append(
        &mut [0x17, 0x72, 0x45, 0x38, 0x50, 0x90]
            .iter()
            .flat_map(|x| convert_to_code_pad_to_byte(*x as u8))
            .collect::<Vec<_>>(),
    );
    let mut crc_as_bits = crc_as_bytes(crc)
        .iter()
        .flat_map(|x| convert_to_code_pad_to_byte(*x))
        .collect::<Vec<_>>();
    out.append(&mut crc_as_bits);
    out
}

pub fn file_header() -> Vec<Bit> {
    let mut out = vec![];
    out.append(&mut convert_to_code_pad_to_byte(b'B'));
    out.append(&mut convert_to_code_pad_to_byte(b'Z'));
    out.append(&mut convert_to_code_pad_to_byte(b'h'));
    out.append(&mut convert_to_code_pad_to_byte(b'9'));
    out
}
