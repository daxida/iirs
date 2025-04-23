// This may present differences in the ordering with IUPACpal - but it is simpler to write

use crate::{config::Config, matrix::MatchMatrix};
use std::fmt::Write;

const fn int_size(x: usize) -> usize {
    (x.ilog10() + 1) as usize
}

/// Follows [IUPACpal](https://github.com/steven31415/IUPACpal) convention
/// of calling Inverted Repeats, palindromes
pub fn fmt_classic_header(config: &Config, n: usize) -> String {
    format!(
        "Palindromes of: {}\n\
        Sequence name: {}\n\
        Sequence length is: {}\n\
        Start at position: {}\n\
        End at position: {}\n\
        Minimum length of Palindromes is: {}\n\
        Maximum length of Palindromes is: {}\n\
        Maximum gap between elements is: {}\n\
        Number of mismatches allowed in Palindrome: {}\n\n\n\n\
        Palindromes:",
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
) -> String {
    let mut out = String::new();

    let pad = "         ";
    let pad_length = pad.len(); // 9

    for &(left, right, gap) in irs {
        let outer_left = left + 1;
        let outer_right = right + 1;
        let inner_left = (outer_left + outer_right - 1 - gap) / 2;
        let inner_right = (outer_right + outer_left + 1 + gap) / 2;

        let ol_pad = " ".repeat(pad_length - int_size(outer_left));
        let il_pad = " ".repeat(pad_length - int_size(inner_left));
        let or_pad = " ".repeat(pad_length - int_size(outer_right));
        let ir_pad = " ".repeat(pad_length - int_size(inner_right));

        // 1. First line (nucleotide strand)
        write!(&mut out, "{}{}", outer_left, ol_pad).unwrap();
        for i in left..inner_left {
            out.push(seq[i] as char);
        }
        write!(&mut out, "{}{}\n", il_pad, inner_left).unwrap();

        // 2. Second line (matching bars)
        out.push_str(pad);
        for i in 0..=(inner_left - outer_left) {
            let l = seq[left + i];
            let r = seq[right - i];
            let matching = matrix.match_u8(l, complement[r as usize]);
            out.push(if matching { '|' } else { ' ' })
        }
        out.push('\n');

        // 3. Third line (reverse complement strand)
        write!(&mut out, "{}{}", outer_right, or_pad).unwrap();
        for i in (inner_right..=outer_right).rev() {
            out.push(seq[i - 1] as char);
        }
        write!(&mut out, "{}{}\n\n", ir_pad, inner_right).unwrap();
    }

    out
}

pub fn fmt_csv_header() -> String {
    String::from("start_n,end_n,nucleotide,start_ir,end_ir,reverse_complement,matching")
}

pub fn fmt_csv(
    irs: &[(usize, usize, usize)],
    seq: &[u8],
    matrix: &MatchMatrix,
    complement: &[u8; 128],
) -> String {
    let mut out = String::new();

    for &(left, right, gap) in irs {
        let outer_left = left + 1;
        let outer_right = right + 1;
        let inner_left = (outer_left + outer_right - 1 - gap) / 2;
        let inner_right = (outer_right + outer_left + 1 + gap) / 2;

        write!(&mut out, "{},{},", outer_left, inner_left).unwrap();

        // 1. Nucleotide strand
        for i in left..inner_left {
            out.push(seq[i] as char);
        }
        out.push(',');

        write!(&mut out, "{},{},", outer_right, inner_right).unwrap();

        // 2. Reverse complement
        for i in (inner_right..=outer_right).rev() {
            out.push(seq[i - 1] as char);
        }
        out.push(',');

        // 3. Matching line
        for i in 0..=(inner_left - outer_left) {
            let l = seq[left + i];
            let r = seq[right - i];
            let matching = matrix.match_u8(l, complement[r as usize]);
            out.push(if matching { '1' } else { '0' });
        }
        out.push('\n');
    }

    out
}

pub fn fmt_custom_header() -> String {
    String::from("ir_start,motif,gap_motif,reverse_complement")
}

pub fn fmt_custom(irs: &[(usize, usize, usize)], seq: &[u8]) -> String {
    let mut out = String::new();

    for &(left, right, gap) in irs {
        let outer_left = left + 1;
        let outer_right = right + 1;
        let inner_left = (outer_left + outer_right - 1 - gap) / 2;
        let inner_right = (outer_right + outer_left + 1 + gap) / 2;

        write!(&mut out, "{},", outer_left).unwrap();

        // 1. Nucleotide strand
        for i in left..inner_left {
            out.push(seq[i] as char);
        }
        out.push(',');

        // 2. Gap motif
        for i in inner_left..(inner_right - 1) {
            out.push(seq[i] as char);
        }
        out.push(',');

        // 3. Reverse complement
        for i in (inner_right..=outer_right).rev() {
            out.push(seq[i - 1] as char);
        }
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SearchParams;
    use crate::{constants::build_complement_array, find_irs, matrix};

    #[test]
    fn test_format_classic() {
        let string = "AGUCSGGTGTWKMMMKKBDDN-NN*HAGNNAGuGTA";
        let seq = string.to_ascii_lowercase().as_bytes().to_vec();
        let params = SearchParams::new(10, 100, 10, 1).unwrap();
        params.check_bounds(seq.len()).unwrap();
        let irs = find_irs(&params, &seq).unwrap();
        let matrix = matrix::MatchMatrix::new();
        let complement = build_complement_array();
        let received = fmt_classic(&irs, &seq, &matrix, &complement);
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
        let irs = find_irs(&params, &seq).unwrap();
        let matrix = matrix::MatchMatrix::new();
        let complement = build_complement_array();
        let received = format!(
            "{}\n{}",
            fmt_csv_header(),
            fmt_csv(&irs, &seq, &matrix, &complement)
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
        let irs = find_irs(&params, &seq).unwrap();
        let received = format!("{}\n{}", fmt_custom_header(), fmt_custom(&irs, &seq));
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
