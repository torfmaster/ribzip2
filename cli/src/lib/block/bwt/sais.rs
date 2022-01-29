#[derive(PartialEq, Eq, Clone)]
enum SuffixType {
    L,
    S,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SuffixTableEntry {
    pub index: usize,
}

/// Build suffix array using the SAIS algorithm
pub(crate) fn build_suffix_array(bytes: &[u8]) -> Vec<SuffixTableEntry> {
    let data = bytes.iter().map(|x| *x as usize).collect::<Vec<usize>>();
    return orchestrate_build_suffix_array(&data, u8::MAX as usize + 1)
        .iter()
        .map(|x| SuffixTableEntry { index: x.unwrap() })
        .collect::<Vec<_>>();
}

fn orchestrate_build_suffix_array(text: &[usize], alphabet_size: usize) -> Vec<Option<usize>> {
    let suffix_types = get_suffix_types(text);
    let bucket_sizes = get_bucket_sizes(text, alphabet_size);
    let n = text.len();
    let mut suffix_array = vec![None; n + 1];

    identify_lms_characters(&mut suffix_array, text, &suffix_types, &bucket_sizes);
    induction_sort_l(&mut suffix_array, text, &suffix_types, &bucket_sizes);
    induction_sort_s(&mut suffix_array, text, &suffix_types, &bucket_sizes);
    let summary = reduce_problem(&mut suffix_array, text, &suffix_types);
    let summary_suffix_array = build_summary_suffix_array(&summary);

    suffix_array = vec![None; n + 1];

    place_lms_characters(
        &mut suffix_array,
        text,
        &bucket_sizes,
        &summary_suffix_array,
        &summary.offsets,
    );
    induction_sort_l(&mut suffix_array, text, &suffix_types, &bucket_sizes);
    induction_sort_s(&mut suffix_array, text, &suffix_types, &bucket_sizes);
    suffix_array
}

fn build_summary_suffix_array(summary: &ReducedProblem) -> Vec<Option<usize>> {
    if summary.alphabet_size == summary.reduced_text.len() {
        let mut suffix_array = vec![Some(0); summary.reduced_text.len() + 1];

        suffix_array[0] = Some(summary.reduced_text.len());
        for i in 1..summary.reduced_text.len() {
            suffix_array[summary.reduced_text[i] + 1] = Some(i);
        }
        return suffix_array;
    }
    orchestrate_build_suffix_array(&summary.reduced_text, summary.alphabet_size)
}

fn reduce_problem(
    suffix_array: &mut [Option<usize>],
    text: &[usize],
    suffix_types: &[SuffixType],
) -> ReducedProblem {
    let mut lms_names = vec![None; text.len() + 1];
    let mut current_name = 0;
    let mut count = 1;
    lms_names[suffix_array[0].unwrap()] = Some(current_name);
    let mut current_offset;
    let mut previous_offset = suffix_array[0];
    for suffix_array_entry in suffix_array[1..].iter() {
        if !is_lms_character(suffix_array_entry.unwrap(), suffix_types) {
            continue;
        };
        current_offset = *suffix_array_entry;
        if !are_lms_blocks_equal(
            text,
            previous_offset.unwrap(),
            current_offset.unwrap(),
            suffix_types,
        ) {
            current_name += 1
        };
        previous_offset = current_offset;
        lms_names[current_offset.unwrap()] = Some(current_name);
        count += 1;
    }
    let mut reduced_text = vec![0usize; count];
    let mut offsets = vec![0usize; count];

    let mut j = 0;
    for (i, &lms_name) in lms_names.iter().enumerate() {
        if lms_name.is_none() {
            continue;
        };
        reduced_text[j] = lms_name.unwrap();
        offsets[j] = i;
        j += 1;
    }
    ReducedProblem {
        reduced_text: reduced_text.into_iter().map(|x| x).collect::<Vec<_>>(),
        offsets,
        alphabet_size: current_name + 1,
    }
}

fn are_lms_blocks_equal(
    text: &[usize],
    previous_offset: usize,
    current_offset: usize,
    suffix_types: &[SuffixType],
) -> bool {
    if previous_offset == text.len() || current_offset == text.len() {
        return false;
    };
    if text[previous_offset] != text[current_offset] {
        return false;
    };
    let mut i = 1;
    while i + current_offset < text.len() && i + previous_offset < text.len() {
        let is_previous_lms = is_lms_character(previous_offset + i, suffix_types);
        let is_current_lms = is_lms_character(current_offset + i, suffix_types);
        if is_previous_lms && is_current_lms {
            return true;
        };
        if is_previous_lms != is_current_lms {
            return false;
        };
        if text[previous_offset + i] != text[current_offset + i] {
            return false;
        };
        i += 1;
    }
    false
}

fn induction_sort_s(
    suffix_array: &mut [Option<usize>],
    text: &[usize],
    suffix_types: &[SuffixType],
    bucket_sizes: &[usize],
) {
    let mut bucket_tails = get_bucket_tails(bucket_sizes);
    for i in (0..suffix_array.len()).rev() {
        let j = suffix_array[i]
            .map(|number| number.checked_sub(1))
            .unwrap_or(None);

        match j {
            Some(j) => {
                if suffix_types.get(j).unwrap() != &SuffixType::S {
                    continue;
                }

                suffix_array[bucket_tails[text[j]]] = Some(j);
                bucket_tails[text[j]] -= 1;
            }
            None => {
                continue;
            }
        }
    }
}

fn induction_sort_l(
    suffix_array: &mut [Option<usize>],
    text: &[usize],
    suffix_types: &[SuffixType],
    bucket_sizes: &[usize],
) {
    let mut bucket_heads = get_bucket_heads(bucket_sizes);
    for i in 0..suffix_array.len() {
        let j = suffix_array[i]
            .map(|number| number.checked_sub(1))
            .unwrap_or(None);
        match j {
            Some(j) => {
                if suffix_types.get(j).unwrap() != &SuffixType::L {
                    continue;
                }
                suffix_array[bucket_heads[text[j]]] = Some(j);
                bucket_heads[text[j]] += 1;
            }
            None => {
                continue;
            }
        }
    }
}

fn identify_lms_characters(
    suffix_array: &mut [Option<usize>],
    text: &[usize],
    suffix_types: &[SuffixType],
    bucket_sizes: &[usize],
) {
    suffix_array[0] = Some(text.len());
    let mut bucket_tails = get_bucket_tails(bucket_sizes);

    for i in (0..text.len()).rev() {
        if !is_lms_character(i, suffix_types) {
            continue;
        };
        suffix_array[bucket_tails[text[i]]] = Some(i);
        bucket_tails[text[i]] -= 1;
    }
}

fn place_lms_characters(
    suffix_array: &mut [Option<usize>],
    text: &[usize],
    bucket_sizes: &[usize],
    summary_suffix_array: &[Option<usize>],
    summary_offsets: &[usize],
) {
    suffix_array[0] = Some(text.len());
    let mut bucket_tails = get_bucket_tails(bucket_sizes);
    for i in (2..(summary_suffix_array.len())).rev() {
        let char_index: usize = summary_offsets[summary_suffix_array[i].unwrap()];
        let bucket_index: usize = text[char_index];
        suffix_array[bucket_tails[bucket_index]] = Some(char_index);
        bucket_tails[bucket_index] -= 1;
    }
}

fn is_lms_character(index: usize, suffix_types: &[SuffixType]) -> bool {
    if index == 0 {
        return false;
    };
    return suffix_types.get(index).unwrap() == &SuffixType::S
        && suffix_types.get(index.checked_sub(1).unwrap()).unwrap() == &SuffixType::L;
}

fn get_bucket_heads(bucket_sizes: &[usize]) -> Vec<usize> {
    let mut heads = vec![0; bucket_sizes.len()];

    let mut offset = 1;
    for i in 0..bucket_sizes.len() {
        heads[i] = offset;
        offset += bucket_sizes[i];
    }
    heads
}

fn get_bucket_tails(bucket_sizes: &[usize]) -> Vec<usize> {
    let mut tails = vec![0; bucket_sizes.len()];
    let mut offset = 1;
    for i in 0..bucket_sizes.len() {
        offset += bucket_sizes[i];
        tails[i] = offset - 1;
    }
    tails
}

fn get_bucket_sizes(text: &[usize], alphabet_size: usize) -> Vec<usize> {
    let mut sizes = vec![0; alphabet_size];
    for character in text {
        sizes[*character] += 1;
    }
    sizes
}

fn get_suffix_types(text: &[usize]) -> Vec<SuffixType> {
    let n = text.len();
    let mut types = vec![SuffixType::L; n + 1];
    types.push(SuffixType::S);

    if n == 0 {
        return types;
    }
    *types.get_mut(n - 1).unwrap() = SuffixType::L;

    for i in (0..n - 1).rev() {
        match text[i].cmp(&text[i + 1]) {
            std::cmp::Ordering::Less => *types.get_mut(i).unwrap() = SuffixType::S,
            std::cmp::Ordering::Equal => {
                *types.get_mut(i).unwrap() = types.get(i + 1).unwrap().clone()
            }
            std::cmp::Ordering::Greater => *types.get_mut(i).unwrap() = SuffixType::L,
        }
    }
    types
}

struct ReducedProblem {
    reduced_text: Vec<usize>,
    offsets: Vec<usize>,
    alphabet_size: usize,
}
