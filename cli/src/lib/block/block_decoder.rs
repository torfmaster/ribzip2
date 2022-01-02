use std::io::Write;

use crate::{bitwise::{bitreader::BitReader, bitwriter::{Bit, convert_to_number}}, lib::block::{bwt::bwt_inverse::inverse_bwt, code_table::ReadDelta, crc32::crc32, huffman::{CodeTable, HuffmanSymbol, reader::ReadSymbols}, mtf::inverse_mtf, rle::inverse_rle, selectors::ReadUnary, symbol_map::GetSymbolTable, zle::{ZleSymbol, decode_zle}}};

pub fn decode_block(mut reader: impl BitReader, mut writer: impl Write) -> Result<(), ()> {
    let crc = convert_to_number(&reader.read_bits(32).unwrap())
        .try_into()
        .unwrap();
    let _randomized = matches!(reader.read_bits(1)?[..], [Bit::One]);
    let orig_ptr = convert_to_number(&reader.read_bits(24).unwrap());
    let symbols = reader.get_symbol_table().unwrap();
    let num_trees = convert_to_number(&reader.read_bits(3).unwrap());
    let num_selectors = convert_to_number(&reader.read_bits(15).unwrap());
    let selectors = reader.read_unary(num_selectors).unwrap();
    let mut trees = vec![];
    for _ in 0..num_trees {
        trees.push(reader.read_delta(symbols.len() + 2).unwrap());
    }
    let mut code_tables = vec![];
    for tree in trees.iter() {
        code_tables.push(CodeTable::<HuffmanSymbol<ZleSymbol>>::from_weights(tree).canonicalize());
    }

    let mut zle_input = vec![];
    for selector in selectors {
        let table = &code_tables[usize::from(selector)];
        zle_input.append(&mut reader.read_symbols(table, 50).unwrap());
    }

    let mtf_input = decode_zle(&zle_input);

    let bwt_input = inverse_mtf(&mtf_input, &symbols);
    let rle_input = inverse_bwt(&bwt_input, orig_ptr);
    let decoded = inverse_rle(&rle_input);
    let computed_crc = crc32(&decoded);
    if computed_crc != crc {
        return Err(());
    }
    writer.write(&decoded).map_err(|_| ())?;
    Ok(())
}

#[cfg(test)]
mod test {}
