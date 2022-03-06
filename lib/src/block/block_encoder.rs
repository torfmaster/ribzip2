use crate::{
    bitwise::bitwriter::{
        convert_to_code_pad_to_15_bits, convert_to_code_pad_to_byte, convert_to_code_pad_to_bytes,
        convert_to_code_pad_to_n_bits,
    },
    {
        bitwise::Bit,
        block::{
            bwt::bwt,
            code_table::encode_code_table,
            crc32::crc32,
            huffman::{compute_huffman, HuffmanSymbol},
            mtf::mtf,
            selectors::create_selectors,
            symbol_map::get_symbol_table,
            zle::zle_transform,
        },
    },
};

use super::symbol_statistics::{
    BlockWisePropabilityMap, EncodingStrategy, ReportedSymbols, SinglePropabilityMap,
};

pub(crate) fn generate_block_data(
    input: &[u8],
    rle_data: Vec<u8>,
    encoding_strategy: EncodingStrategy,
) -> (Vec<Bit>, u32) {
    let mut output = Vec::<Bit>::new();
    let checksum = crc32(input);

    let bwt_data = bwt(rle_data);
    let mtf_data = mtf(&bwt_data.data);
    let (zle_data, frequencies) = match encoding_strategy {
        EncodingStrategy::BlockWise {
            num_clusters,
            num_iterations,
        } => zle_transform(
            mtf_data.encoded,
            BlockWisePropabilityMap::create(
                mtf_data.used_symbols.len(),
                num_clusters,
                num_iterations,
            ),
        ),
        EncodingStrategy::Single => zle_transform(
            mtf_data.encoded,
            SinglePropabilityMap::create(mtf_data.used_symbols.len()),
        ),
    };

    let ReportedSymbols {
        reported_frequencies,
        selectors: selected_tables,
    } = frequencies;
    let code_tables = reported_frequencies
        .into_iter()
        .map(|table| compute_huffman(table).canonicalize())
        .collect::<Vec<_>>();
    let mut selectors = create_selectors(&selected_tables);
    let num_tables = code_tables.len();
    let mut symbol_map = get_symbol_table(mtf_data.used_symbols);

    let trees = code_tables
        .iter()
        .cloned()
        .map(|code_table| encode_code_table(code_table))
        .collect::<Vec<_>>();

    // block
    output.append(&mut block_header(checksum, bwt_data.end_of_string as u32));
    output.append(&mut symbol_map);

    output.append(&mut &mut convert_to_code_pad_to_n_bits(num_tables, 3));

    output.append(&mut convert_to_code_pad_to_15_bits(selectors.1 as u16));

    // the selectors (/)
    output.append(&mut selectors.0);

    // write trees
    for mut tree in trees.iter().cloned() {
        output.append(&mut tree)
    }

    // data
    for (position, symbol) in zle_data.iter().enumerate() {
        let symbol = HuffmanSymbol::NormalSymbol(symbol.clone());
        // get correct table
        let code_table = &code_tables[selected_tables[position / 50] as usize];

        for table_entry in code_table.0.iter() {
            if table_entry.symbol == symbol {
                output.append(&mut table_entry.code.clone());
                break;
            }
        }
    }

    let position = zle_data.len();
    let code_table = &code_tables[selected_tables[position / 50] as usize];

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
