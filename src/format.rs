// This may present differences in the ordering with IUPACpal - but it is simpler to write

use crate::{config::Config, matrix::MatchMatrix};

fn int_size(x: i32) -> usize {
    format!("{}", x).len()
}

pub fn out_palindrome_display_header(config: &Config, n: usize) -> String {
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
        Palindromes:\n",
        &config.input_file,
        &config.seq_name,
        n,
        1,
        n,
        config.min_len,
        config.max_len,
        config.max_gap,
        config.mismatches,
    )
}

pub fn fmt_classic(
    palindromes: &Vec<(i32, i32, i32)>,
    seq: &[u8],
    matrix: &MatchMatrix,
    complement: &[u8; 128],
) -> String {
    let mut palindromes_out = String::new();

    let pad = "         ";
    let pad_length = pad.len();

    for (left, right, gap) in palindromes {
        let outer_left = left + 1;
        let outer_right = right + 1;
        let inner_left = (outer_left + outer_right - 1 - gap) / 2;
        let inner_right = (outer_right + outer_left + 1 + gap) / 2;

        let first_line = format!(
            "{}{}{}{}{}\n",
            outer_left,
            " ".repeat(pad_length - int_size(outer_left)),
            (outer_left..=inner_left)
                .map(|i| seq[(i - 1) as usize] as char)
                // (*left as usize..inner_left as usize)
                //     .map(|i| seq[i] as char)
                .collect::<String>(),
            " ".repeat(pad_length - int_size(inner_left)),
            inner_left,
        );

        let matching_line = format!(
            "{}{}\n",
            pad,
            (0..=(inner_left - outer_left))
                .map(|i| {
                    let l = seq[(left + i) as usize];
                    let r = seq[(right - i) as usize];
                    if matrix.match_u8(l, complement[r as usize]) {
                        "|"
                    } else {
                        " "
                    }
                })
                .collect::<String>()
        );

        let second_line = format!(
            "{}{}{}{}{}\n\n",
            outer_right,
            " ".repeat(pad_length - int_size(outer_right)),
            (inner_right..=outer_right)
                .rev()
                .map(|i| seq[(i - 1) as usize] as char)
                .collect::<String>(),
            " ".repeat(pad_length - int_size(inner_right)),
            inner_right,
        );

        palindromes_out.push_str(&format!("{}{}{}", first_line, matching_line, second_line));
    }

    palindromes_out
}

pub fn fmt_csv(
    palindromes: &Vec<(i32, i32, i32)>,
    seq: &[u8],
    matrix: &MatchMatrix,
    complement: &[u8; 128],
) -> String {
    let mut palindromes_out = String::new();

    let heading = "start_n,end_n,nucleotide,start_ir,end_ir,reverse_complement,matching\n";
    palindromes_out.push_str(heading);

    for (left, right, gap) in palindromes {
        let outer_left = left + 1;
        let outer_right = right + 1;
        let inner_left = (outer_left + outer_right - 1 - gap) / 2;
        let inner_right = (outer_right + outer_left + 1 + gap) / 2;

        let nucleotide = (*left as usize..inner_left as usize)
            .map(|i| seq[i] as char)
            .collect::<String>();
        let reverse_complement = ((inner_right - 1) as usize..outer_right as usize)
            .rev()
            .map(|i| seq[i] as char)
            .collect::<String>();
        let matching_line = (0..=(inner_left - outer_left))
            .map(|i| {
                let l = seq[(left + i) as usize];
                let r = seq[(right - i) as usize];
                if matrix.match_u8(l, complement[r as usize]) {
                    "1"
                } else {
                    "0"
                }
            })
            .collect::<String>();

        palindromes_out.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            outer_left,
            inner_left,
            nucleotide,
            outer_right,
            inner_right,
            reverse_complement,
            matching_line
        ));
    }

    palindromes_out
}

pub fn fmt_custom(palindromes: &Vec<(i32, i32, i32)>, seq: &[u8]) -> String {
    let mut palindromes_out = String::new();

    let heading = "ir_start,motif,gap_motif,reverse_complement\n";
    palindromes_out.push_str(heading);

    for (left, right, gap) in palindromes {
        let outer_left = left + 1;
        let outer_right = right + 1;
        let inner_left = (outer_left + outer_right - 1 - gap) / 2;
        let inner_right = (outer_right + outer_left + 1 + gap) / 2;

        let nucleotide = (*left as usize..inner_left as usize)
            .map(|i| seq[i] as char)
            .collect::<String>();
        let gap_nucleotide = (inner_left as usize..(inner_right - 1) as usize)
            .map(|i| seq[i] as char)
            .collect::<String>();
        let reverse_complement = ((inner_right - 1) as usize..outer_right as usize)
            .rev()
            .map(|i| seq[i] as char)
            .collect::<String>();

        palindromes_out.push_str(&format!(
            "{},{},{},{}\n",
            outer_left, nucleotide, gap_nucleotide, reverse_complement
        ));
    }

    palindromes_out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{constants::build_complement_array, find_palindromes, matrix};

    #[test]
    fn test_format_classic() {
        let config = Config::dummy(10, 100, 10, 1);
        let string = "AGUCSGGTGTWKMMMKKBDDN-NN*HAGNNAGuGTA";
        let seq = string.to_ascii_lowercase().as_bytes().to_vec();
        let n = seq.len();
        let _ = config.verify(n).unwrap();
        let palindromes = find_palindromes(&config, &seq);
        let matrix = matrix::MatchMatrix::new();
        let complement = build_complement_array();
        let received = fmt_classic(&palindromes, &seq, &matrix, &complement);
        let expected = r#"2        gucsggtgtwkmmm       15
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
33       uganngah*n       24"#;
        let expected_lines = expected.split("\n");
        let received_lines = received.split("\n");
        for (idx, (e, r)) in expected_lines.zip(received_lines).enumerate() {
            assert_eq!(e, r, "Difference at line {}", idx)
        }
    }

    #[test]
    fn test_format_csv() {
        let config = Config::dummy(10, 100, 10, 1);
        let string = "AGUCSGGTGTWKMMMKKBDDN-NN*HAGNNAGuGTA";
        let seq = string.to_ascii_lowercase().as_bytes().to_vec();
        let n = seq.len();
        let _ = config.verify(n).unwrap();
        let palindromes = find_palindromes(&config, &seq);
        let matrix = matrix::MatchMatrix::new();
        let complement = build_complement_array();
        let received = fmt_csv(&palindromes, &seq, &matrix, &complement);
        let expected = r#"start_n,end_n,nucleotide,start_ir,end_ir,reverse_complement,matching
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
"#;
        let expected_lines = expected.split("\n");
        let received_lines = received.split("\n");
        for (idx, (e, r)) in expected_lines.zip(received_lines).enumerate() {
            assert_eq!(e, r, "Difference at line {}", idx)
        }
    }

    #[test]
    fn test_format_custom_csv_mini() {
        let config = Config::dummy(10, 100, 10, 1);
        let string = "AGUCSGGTGTWKMMMKKBDDN-NN*HAGNNAGuGTA";
        let seq = string.to_ascii_lowercase().as_bytes().to_vec();
        let n = seq.len();
        let _ = config.verify(n).unwrap();
        let palindromes = find_palindromes(&config, &seq);
        let received = fmt_custom(&palindromes, &seq);
        let expected = r#"ir_start,motif,gap_motif,reverse_complement
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
"#;
        let expected_lines = expected.split("\n");
        let received_lines = received.split("\n");
        for (idx, (e, r)) in expected_lines.zip(received_lines).enumerate() {
            assert_eq!(e, r, "Difference at line {}", idx)
        }
    }
}
