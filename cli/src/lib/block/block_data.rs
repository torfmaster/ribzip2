use crate::{bitwise::bitwriter::{Bit, convert_to_code_pad_to_15_bits, convert_to_code_pad_to_byte, convert_to_code_pad_to_bytes}, lib::block::{bwt::bwt, code_table::encode_code_table, crc32::crc32, huffman::{HuffmanSymbol, compute_huffman}, mtf::mtf, rle::rle, selectors::create_selectors, symbol_map::get_symbol_table, zle::zle_transform}};

pub fn generate_block_data(input: &[u8]) -> (Vec<Bit>, u32) {
    let mut output = Vec::<Bit>::new();
    let checksum = crc32(input);

    let rle_data = rle(input);
    let bwt_data = bwt(rle_data);
    let mtf_data = mtf(&bwt_data.data);
    let (zle_data, frequencies) = zle_transform(mtf_data.encoded, mtf_data.used_symbols.len());

    let code_table = compute_huffman(frequencies).canonicalize();
    let mut selectors = create_selectors(zle_data.len() + 1);
    let mut symbol_map = get_symbol_table(mtf_data.used_symbols);
    let mut tree = encode_code_table(code_table.clone());

    // block
    output.append(&mut block_header(checksum, bwt_data.end_of_string as u32));
    output.append(&mut symbol_map);

    output.append(&mut vec![Bit::Zero, Bit::One, Bit::Zero]); // numtress (hard coded to 2)

    output.append(&mut convert_to_code_pad_to_15_bits(selectors.1 as u16));

    // the selectors (/)
    output.append(&mut selectors.0);

    // Duplicate tree
    output.append(&mut tree.clone());
    output.append(&mut tree);

    // data
    for symbol in zle_data.iter() {
        let symbol = HuffmanSymbol::NormalSymbol(symbol.clone());
        for table_entry in code_table.0.iter() {
            if table_entry.symbol == symbol {
                output.append(&mut table_entry.code.clone());
                break;
            }
        }
    }
    // write eob marker
    output.append(
        &mut code_table
            .0
            .iter()
            .find(|x| x.symbol == HuffmanSymbol::EoB)
            .unwrap()
            .code
            .clone(),
    );
    (output, checksum)
}

pub(crate) fn block_header(crc: u32, orig_pointer: u32) -> Vec<Bit> {
    let mut magic: Vec<Bit> = vec![0x31u8, 0x41u8, 0x59u8, 0x26u8, 0x53u8, 0x59u8]
        .iter()
        .flat_map(|x| convert_to_code_pad_to_byte(*x))
        .collect::<Vec<_>>();
    magic.append(&mut convert_to_code_pad_to_bytes(&crc_as_bytes(crc)));
    magic.push(Bit::Zero); // randomized: false

    magic.append(&mut convert_to_code_pad_to_byte(
        ((orig_pointer & 0xFF0000) >> 16) as u8,
    ));
    magic.append(&mut convert_to_code_pad_to_byte(
        ((orig_pointer & 0xFF00) >> 8) as u8,
    ));
    magic.append(&mut convert_to_code_pad_to_byte(
        (orig_pointer & 0xFF) as u8,
    ));
    magic
}

pub fn crc_as_bytes(mut crc: u32) -> Vec<u8> {
    let mut magic = Vec::new();
    for _ in 0..4 {
        magic.push((crc & 0xFF) as u8);
        crc >>= 8;
    }
    magic.reverse();

    magic
}
