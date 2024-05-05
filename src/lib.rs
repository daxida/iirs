pub mod config;

mod algo;
mod constants;
mod format;
mod matrix;
mod rmq;
mod utils;

use anyhow::Result;
use config::{Config, Parameters};

/// Find palindromes in a sequence based on the provided configuration.
///
/// Each palindrome is a tuple of three integers (usize): start position, end position, and gap size.
///
/// # Examples
///
/// ```rust
/// use iupacpal::{config::Parameters, find_palindromes};
///
/// let seq = "acbbgt".as_bytes();
/// let params = Parameters::new(3, 6, 2, 0);
/// assert!(params.verify_bounds(seq.len()).is_ok());
/// let palindromes = find_palindromes(&params, &seq);
/// assert_eq!(palindromes.unwrap(), vec![(0, 5, 0)]);
///
/// // Returns an error if the given sequence contains invalid characters
/// let seq = "jj".as_bytes();
/// let palindromes = find_palindromes(&params, &seq);
/// assert!(palindromes.is_err());
///
/// // It is not case-sensitive and ignores newlines.
/// let seq = "ACB\n\rBGT".as_bytes();
/// let palindromes = find_palindromes(&params, &seq);
/// assert_eq!(palindromes.unwrap(), vec![(0, 5, 0)]);
/// ```
#[elapsed_time::elapsed]
pub fn find_palindromes(params: &Parameters, seq: &[u8]) -> Result<Vec<(usize, usize, usize)>> {
    // Removes newlines, cast to lowercase and checks that all the character are in IUPAC.
    // This was already done through the CLI, but we need to do it again for the standalone version.
    let sanitized_seq = utils::sanitize_sequence(seq)?;

    // Build matchmatrix
    let matrix = matrix::MatchMatrix::new();
    let complement = constants::build_complement_array();

    // Construct s = seq + '$' + complement(reverse(seq)) + '#'
    let n = sanitized_seq.len();
    let s_n = 2 * n + 2;
    let mut s = vec![0u8; s_n];
    for i in 0..n {
        s[i] = sanitized_seq[i];
        s[n + 1 + i] = complement[sanitized_seq[n - 1 - i] as usize] as u8;
    }
    s[n] = b'$';
    s[2 * n + 1] = b'#';

    // Construct Suffix Array (sa) & Inverse Suffix Array
    let sa: Vec<i32> = divsufsort::sort(&s).into_parts().1;
    let mut inv_sa = vec![0; s_n];
    for (i, value) in sa.iter().enumerate() {
        inv_sa[*value as usize] = i;
    }

    // Calculate LCP & RMQ
    let lcp = algo::lcp_array(&s, s_n, &sa, &inv_sa);
    let rmq = rmq::Sparse::new(&lcp);

    // Calculate palidromes
    let mut palindromes = algo::add_palindromes(&s, &inv_sa, &rmq, params, &matrix);

    // Deal with the sorting strategy.
    // Alternatives, or even skipping sorting altogether, can improve the performance.
    // The original IUPACpal sorts by (left, gap_size, -right)
    palindromes.sort_by(|a, b| {
        let cmp_left = a.0.cmp(&b.0);
        let cmp_gap = a.2.cmp(&a.2);
        let cmp_right = b.1.cmp(&a.1);
        cmp_left.then(cmp_gap).then(cmp_right)
    });

    Ok(palindromes)
}

/// Stringify the given palindromes according to the configuration output format.
///
/// Returns an error if given an invalid output format.
///
/// # Examples
///
/// ```rust
/// use iupacpal::{config::Config, find_palindromes, stringify_palindromes};
///
/// let seq = "acbbgt".as_bytes();
/// let config = Config::new("in.fasta", "seq0", 3, 6, 2, 0, "out.txt", "csv");
/// let palindromes = find_palindromes(&config.parameters, &seq).unwrap();
/// let out_str = stringify_palindromes(&config, &palindromes, &seq).unwrap();
/// let expected = "\
///     start_n,end_n,nucleotide,start_ir,end_ir,reverse_complement,matching\n\
///     1,3,acb,6,4,tgb,111\n";
///
/// assert_eq!(out_str, expected);
/// ```
pub fn stringify_palindromes(
    config: &Config,
    palindromes: &Vec<(usize, usize, usize)>,
    seq: &[u8],
) -> Result<String> {
    let matrix = matrix::MatchMatrix::new();
    let complement = constants::build_complement_array();

    utils::verify_format(&config.output_format)?;

    match config.output_format.as_str() {
        "classic" => Ok(format!(
            "{}{}",
            format::out_palindrome_display_header(config, seq.len()),
            format::fmt_classic(palindromes, seq, &matrix, &complement)
        )),
        "csv" => Ok(format::fmt_csv(palindromes, seq, &matrix, &complement)),
        "custom" => Ok(format::fmt_custom(palindromes, seq)),
        // Already tested in utils::verify_format
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod libtests;
