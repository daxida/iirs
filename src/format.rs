// This may present differences in the ordering with IUPACpal - but it is simpler to write
#![allow(clippy::needless_range_loop)]
#![allow(clippy::similar_names)]

use crate::{config::Config, constants::RepeatType, matrix::MatchMatrix};
use std::fmt::Write;

const fn int_size(x: usize) -> usize {
    (x.ilog10() + 1) as usize
}

/// The two arms of a repeat, as 1-based inclusive positions.
///
/// ```text
/// [ first arm ] ... gap ... [ second arm ]
/// ^           ^             ^            ^
/// outer_left  inner_left    inner_right  outer_right
/// ```
///
/// The second arm is read from `outer_right` down to `inner_right` when the repeat is
/// reversed, and from `inner_right` up to `outer_right` otherwise.
struct Arms {
    outer_left: usize,
    inner_left: usize,
    inner_right: usize,
    outer_right: usize,
    arm_len: usize,
    repeat_type: RepeatType,
}

impl Arms {
    const fn new((left, right, gap): (usize, usize, usize), repeat_type: RepeatType) -> Self {
        let outer_left = left + 1;
        let outer_right = right + 1;

        Self {
            outer_left,
            inner_left: (outer_left + outer_right - 1 - gap) / 2,
            inner_right: (outer_right + outer_left + 1 + gap) / 2,
            outer_right,
            arm_len: (outer_right + 1 - outer_left - gap) / 2,
            repeat_type,
        }
    }

    /// 0-based index in `seq` of the `i`th character of the first arm.
    const fn first(&self, i: usize) -> usize {
        self.outer_left - 1 + i
    }

    /// 0-based index in `seq` of the character the `i`th one of the first arm pairs with.
    const fn second(&self, i: usize) -> usize {
        if self.repeat_type.is_reversed() {
            self.outer_right - 1 - i
        } else {
            self.inner_right - 1 + i
        }
    }

    /// Labels of the first and last printed character of the second arm.
    const fn second_bounds(&self) -> (usize, usize) {
        if self.repeat_type.is_reversed() {
            (self.outer_right, self.inner_right)
        } else {
            (self.inner_right, self.outer_right)
        }
    }

    /// Whether the `i`th pair of the repeat matches.
    fn matches(&self, i: usize, seq: &[u8], matrix: &MatchMatrix, complement: &[u8; 128]) -> bool {
        let l = seq[self.first(i)];
        let r = seq[self.second(i)];
        let r = if self.repeat_type.is_complemented() {
            complement[r as usize]
        } else {
            r
        };

        matrix.match_u8(l, r)
    }
}

/// Follows [IUPACpal](https://github.com/steven31415/IUPACpal) convention
/// of calling Inverted Repeats, palindromes
pub fn fmt_classic_header(config: &Config, n: usize) -> String {
    let noun = config.params.repeat_type.noun();

    format!(
        "{noun}s of: {}\n\
        Sequence name: {}\n\
        Sequence length is: {}\n\
        Start at position: {}\n\
        End at position: {}\n\
        Minimum length of {noun}s is: {}\n\
        Maximum length of {noun}s is: {}\n\
        Maximum gap between elements is: {}\n\
        Number of mismatches allowed in {noun}: {}\n\n\n\n\
        {noun}s:",
        &config.input_file,
        &config.seq_name,
        n,
        1,
        n,
        config.params.min_len,
        config.params.max_len,
        config.params.max_gap,
        config.params.mismatches,
    )
}

pub fn fmt_classic(
    irs: &[(usize, usize, usize)],
    seq: &[u8],
    matrix: &MatchMatrix,
    complement: &[u8; 128],
    repeat_type: RepeatType,
) -> String {
    let mut out = String::new();

    let pad = "         ";
    let pad_length = pad.len(); // 9

    for &ir in irs {
        let arms = Arms::new(ir, repeat_type);
        let (second_from, second_to) = arms.second_bounds();

        let ol_pad = " ".repeat(pad_length - int_size(arms.outer_left));
        let il_pad = " ".repeat(pad_length - int_size(arms.inner_left));
        let sf_pad = " ".repeat(pad_length - int_size(second_from));
        let st_pad = " ".repeat(pad_length - int_size(second_to));

        // 1. First line (nucleotide strand)
        write!(&mut out, "{}{ol_pad}", arms.outer_left).unwrap();
        for i in 0..arms.arm_len {
            out.push(seq[arms.first(i)] as char);
        }
        writeln!(&mut out, "{il_pad}{}", arms.inner_left).unwrap();

        // 2. Second line (matching bars)
        out.push_str(pad);
        for i in 0..arms.arm_len {
            let matching = arms.matches(i, seq, matrix, complement);
            out.push(if matching { '|' } else { ' ' });
        }
        out.push('\n');

        // 3. Third line (second strand)
        write!(&mut out, "{second_from}{sf_pad}").unwrap();
        for i in 0..arms.arm_len {
            out.push(seq[arms.second(i)] as char);
        }
        write!(&mut out, "{st_pad}{second_to}\n\n").unwrap();
    }

    out
}

pub fn fmt_csv_header(repeat_type: RepeatType) -> String {
    format!(
        "start_n,end_n,nucleotide,start_ir,end_ir,{},matching",
        repeat_type.arm_label()
    )
}

pub fn fmt_csv(
    irs: &[(usize, usize, usize)],
    seq: &[u8],
    matrix: &MatchMatrix,
    complement: &[u8; 128],
    repeat_type: RepeatType,
) -> String {
    let mut out = String::new();

    for &ir in irs {
        let arms = Arms::new(ir, repeat_type);
        let (second_from, second_to) = arms.second_bounds();

        write!(&mut out, "{},{},", arms.outer_left, arms.inner_left).unwrap();

        // 1. Nucleotide strand
        for i in 0..arms.arm_len {
            out.push(seq[arms.first(i)] as char);
        }
        out.push(',');

        write!(&mut out, "{second_from},{second_to},").unwrap();

        // 2. Second strand
        for i in 0..arms.arm_len {
            out.push(seq[arms.second(i)] as char);
        }
        out.push(',');

        // 3. Matching line
        for i in 0..arms.arm_len {
            let matching = arms.matches(i, seq, matrix, complement);
            out.push(if matching { '1' } else { '0' });
        }
        out.push('\n');
    }

    out
}

pub fn fmt_custom_header(repeat_type: RepeatType) -> String {
    format!("ir_start,motif,gap_motif,{}", repeat_type.arm_label())
}

pub fn fmt_custom(irs: &[(usize, usize, usize)], seq: &[u8], repeat_type: RepeatType) -> String {
    let mut out = String::new();

    for &ir in irs {
        let arms = Arms::new(ir, repeat_type);

        write!(&mut out, "{},", arms.outer_left).unwrap();

        // 1. Nucleotide strand
        for i in 0..arms.arm_len {
            out.push(seq[arms.first(i)] as char);
        }
        out.push(',');

        // 2. Gap motif
        for i in arms.inner_left..(arms.inner_right - 1) {
            out.push(seq[i] as char);
        }
        out.push(',');

        // 3. Second strand
        for i in 0..arms.arm_len {
            out.push(seq[arms.second(i)] as char);
        }
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SearchParams;
    use crate::{constants::build_complement_array, find_repeats, matrix};

    #[test]
    fn test_format_classic() {
        let string = "AGUCSGGTGTWKMMMKKBDDN-NN*HAGNNAGuGTA";
        let seq = string.to_ascii_lowercase().as_bytes().to_vec();
        let params = SearchParams::new(10, 100, 10, 1).unwrap();
        params.check_bounds(seq.len()).unwrap();
        let irs = find_repeats(&params, &seq).unwrap();
        let matrix = matrix::MatchMatrix::new();
        let complement = build_complement_array();
        let received = fmt_classic(&irs, &seq, &matrix, &complement, RepeatType::Inverted);
        let expected = r"2        gucsggtgtwkmmm       15
         ||| ||||||||||
30       nngah*nn-nddbk       17

3        ucsggtgtwkmmm       15
         ||| |||||||||
30       nngah*nn-nddb       18

3        ucsggtgtwkmm       14
         | ||||||||||
27       ah*nn-nddbkk       16

5        sggtgtwkmmmkk       17
         || ||||||||||
30       nngah*nn-nddb       18

5        sggtgtwkmmm       15
         |||||||||||
26       h*nn-nddbkk       16

7        gtgtwkmmmkkb       18
         || |||||||||
30       nngah*nn-ndd       19

8        tgtwkmmmkkbd       19
         ||| ||||||||
31       anngah*nn-nd       20

8        tgtwkmmmkkb       18
         || ||||||||
30       nngah*nn-nd       20

10       twkmmmkkbdd       20
         |||| ||||||
31       anngah*nn-n       21

11       wkmmmkkbdd       20
         |||| |||||
31       anngah*nn-       22

12       kmmmkkbddn       21
         ||||||||||
31       anngah*nn-       22

13       mmmkkbddn-n       23
         |||||| ||||
34       guganngah*n       24

13       mmmkkbddn-       22
         || |||||||
33       uganngah*n       24";
        let expected_lines = expected.split('\n');
        let received_lines = received.split('\n');
        for (idx, (e, r)) in expected_lines.zip(received_lines).enumerate() {
            assert_eq!(e, r, "Difference at line {idx}");
        }
    }

    #[test]
    fn test_format_csv() {
        let string = "AGUCSGGTGTWKMMMKKBDDN-NN*HAGNNAGuGTA";
        let seq = string.to_ascii_lowercase().as_bytes().to_vec();
        let params = SearchParams::new(10, 100, 10, 1).unwrap();
        params.check_bounds(seq.len()).unwrap();
        let irs = find_repeats(&params, &seq).unwrap();
        let matrix = matrix::MatchMatrix::new();
        let complement = build_complement_array();
        let received = format!(
            "{}\n{}",
            fmt_csv_header(RepeatType::Inverted),
            fmt_csv(&irs, &seq, &matrix, &complement, RepeatType::Inverted)
        );
        let expected = r"start_n,end_n,nucleotide,start_ir,end_ir,reverse_complement,matching
2,15,gucsggtgtwkmmm,30,17,nngah*nn-nddbk,11101111111111
3,15,ucsggtgtwkmmm,30,18,nngah*nn-nddb,1110111111111
3,14,ucsggtgtwkmm,27,16,ah*nn-nddbkk,101111111111
5,17,sggtgtwkmmmkk,30,18,nngah*nn-nddb,1101111111111
5,15,sggtgtwkmmm,26,16,h*nn-nddbkk,11111111111
7,18,gtgtwkmmmkkb,30,19,nngah*nn-ndd,110111111111
8,19,tgtwkmmmkkbd,31,20,anngah*nn-nd,111011111111
8,18,tgtwkmmmkkb,30,20,nngah*nn-nd,11011111111
10,20,twkmmmkkbdd,31,21,anngah*nn-n,11110111111
11,20,wkmmmkkbdd,31,22,anngah*nn-,1111011111
12,21,kmmmkkbddn,31,22,anngah*nn-,1111111111
13,23,mmmkkbddn-n,34,24,guganngah*n,11111101111
13,22,mmmkkbddn-,33,24,uganngah*n,1101111111
";
        let expected_lines = expected.split('\n');
        let received_lines = received.split('\n');
        for (idx, (e, r)) in expected_lines.zip(received_lines).enumerate() {
            assert_eq!(e, r, "Difference at line {idx}");
        }
    }

    #[test]
    fn test_format_custom_csv_mini() {
        let string = "AGUCSGGTGTWKMMMKKBDDN-NN*HAGNNAGuGTA";
        let seq = string.to_ascii_lowercase().as_bytes().to_vec();
        let params = SearchParams::new(10, 100, 10, 1).unwrap();
        params.check_bounds(seq.len()).unwrap();
        let irs = find_repeats(&params, &seq).unwrap();
        let received = format!(
            "{}\n{}",
            fmt_custom_header(RepeatType::Inverted),
            fmt_custom(&irs, &seq, RepeatType::Inverted)
        );
        let expected = r"ir_start,motif,gap_motif,reverse_complement
2,gucsggtgtwkmmm,k,nngah*nn-nddbk
3,ucsggtgtwkmmm,kk,nngah*nn-nddb
3,ucsggtgtwkmm,m,ah*nn-nddbkk
5,sggtgtwkmmmkk,,nngah*nn-nddb
5,sggtgtwkmmm,,h*nn-nddbkk
7,gtgtwkmmmkkb,,nngah*nn-ndd
8,tgtwkmmmkkbd,,anngah*nn-nd
8,tgtwkmmmkkb,d,nngah*nn-nd
10,twkmmmkkbdd,,anngah*nn-n
11,wkmmmkkbdd,n,anngah*nn-
12,kmmmkkbddn,,anngah*nn-
13,mmmkkbddn-n,,guganngah*n
13,mmmkkbddn-,n,uganngah*n
";
        let expected_lines = expected.split('\n');
        let received_lines = received.split('\n');
        for (idx, (e, r)) in expected_lines.zip(received_lines).enumerate() {
            assert_eq!(e, r, "Difference at line {idx}");
        }
    }
}
