use std::collections::BTreeSet; // Ordered

use crate::{
    config::Config,
    matrix::{self, MatchMatrix},
};

type Palindrome = (i32, i32, i32);

pub fn fmt(
    config: &Config,
    palindromes: &BTreeSet<Palindrome>,
    seq: Vec<u8>,
    n: usize,
    matrix: &MatchMatrix,
    complement: &[char; 128],
) -> String {
    let config_out = Config::out_palindrome_display(&config, n);

    // This may present differences in the ordering with IUPACpal - but it is simpler to write
    fn int_size(x: i32) -> usize {
        format!("{}", x).len()
    }

    let mut palindromes_out = String::new();

    for (left, right, gap) in palindromes {
        let outer_left = left + 1;
        let outer_right = right + 1;
        let inner_left = (outer_left + outer_right - 1 - gap) / 2;
        let inner_right = (outer_right + outer_left + 1 + gap) / 2;

        let pad = "         ";
        let pad_length = pad.len();

        let first_line = format!(
            "{}{}{}{}{}\n",
            outer_left,
            " ".repeat(pad_length - int_size(outer_left)),
            (outer_left..=inner_left)
                .map(|i| seq.get((i - 1) as usize).copied().unwrap_or(b' '))
                .map(|b| b as char)
                .collect::<String>(),
            " ".repeat(pad_length - int_size(inner_left)),
            inner_left,
        );

        let matching_line = format!(
            "{}{}\n",
            pad,
            (0..=(inner_left - outer_left))
                .map(|i| {
                    let left  = seq[(outer_left - 1 + i) as usize] as char;
                    let right = seq[(outer_right - 1 - i) as usize] as char;
                    if matrix.match_chars(left, complement[right as usize])  {
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
                .map(|i| seq.get((i - 1) as usize).copied().unwrap_or(b' '))
                .map(|b| b as char)
                .collect::<String>(),
            " ".repeat(pad_length - int_size(inner_right)),
            inner_right,
        );

        palindromes_out += format!("{}{}{}", first_line, matching_line, second_line).as_str();
    }

    format!("{}{}", config_out, palindromes_out)
}
