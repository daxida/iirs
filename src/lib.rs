pub mod config;

mod algo;
mod constants;
mod format;
mod matrix;
mod rmq;

use anyhow::{anyhow, Result};
use config::Config;
use libdivsufsort_rs::*;

/// Find palindromes in a fasta sequence based on the provided configuration.
///
/// Each tuple contains three integers: start position, end position, and gap size.
///
/// # Examples
///
/// ```rust
/// use iupacpal::{config::Config, find_palindromes};
///
/// let seq = "acbbgt".as_bytes();
/// let config = Config::dummy(3, 6, 2, 0);
/// let palindromes = find_palindromes(&config, &seq);
///
/// assert_eq!(palindromes, vec![(0, 5, 0)])
/// ```
#[elapsed_time::elapsed]
pub fn find_palindromes(config: &Config, seq: &[u8]) -> Vec<(i32, i32, i32)> {
    // This recomputation of n is just for convenience of the API
    let n = seq.len();

    // Build matchmatrix
    let matrix = matrix::MatchMatrix::new();
    let complement = constants::build_complement_array();

    // Construct s = seq + '$' + complement(reverse(seq)) + '#'
    let s_n = 2 * n + 2;
    let mut s = vec![0u8; s_n];
    for i in 0..n {
        s[i] = seq[i];
        s[n + 1 + i] = complement[seq[n - 1 - i] as usize] as u8;
    }
    s[n] = b'$';
    s[2 * n + 1] = b'#';

    // Construct Suffix Array (sa) & Inverse Suffix Array
    let sa: Vec<i64> = divsufsort64(&s).unwrap();
    let mut inv_sa = vec![0; s_n];
    for (i, value) in sa.iter().enumerate() {
        inv_sa[*value as usize] = i;
    }

    // Calculate LCP & RMQ
    let lcp = algo::lcp_array(&s, s_n, &sa, &inv_sa);
    let rmq = rmq::Sparse::new(&lcp);

    // Calculate palidromes
    let mut palindromes = algo::add_palindromes(
        &s,
        s_n,
        n,
        &inv_sa,
        &rmq,
        config.min_len as i32,
        config.max_len as i32,
        config.mismatches as i32,
        config.max_gap as i32,
        &matrix,
    );

    // Deal with the sorting strategy.
    // Alternatives, or even skipping sorting altogether, can improve the performance.
    // The original IUPACpal sorts by (left, gap_size, -right)
    palindromes.sort_by(|a, b| {
        let cmp_left = a.0.cmp(&b.0);
        let cmp_gap = a.2.cmp(&a.2);
        let cmp_right = b.1.cmp(&a.1);
        cmp_left.then(cmp_gap).then(cmp_right)
    });

    palindromes
}

/// Stringify the given palindromes according to the configuration output format.
///
/// Returns an error if given an invalid output format.
///
/// # Examples
///
/// ```rust
/// use iupacpal::{config::Config, find_palindromes, strinfigy_palindromes};
///
/// let seq = "acbbgt".as_bytes();
/// let config = Config::new("in.fasta", "seq0", 3, 6, 2, 0, "out.txt", "csv");
/// let palindromes = find_palindromes(&config, &seq);
/// let out_str = strinfigy_palindromes(&config, &palindromes, &seq).unwrap();
/// let expected = "\
///     start_n,end_n,nucleotide,start_ir,end_ir,reverse_complement,matching\n\
///     1,3,acb,6,4,tgb,111\n";
///
/// assert_eq!(out_str, expected);
/// ```
pub fn strinfigy_palindromes(
    config: &Config,
    palindromes: &Vec<(i32, i32, i32)>,
    seq: &[u8],
) -> Result<String> {
    let matrix = matrix::MatchMatrix::new();
    let complement = constants::build_complement_array();

    match config.output_format.as_str() {
        "classic" => Ok(format!(
            "{}{}",
            format::out_palindrome_display_header(config, seq.len()),
            format::fmt_classic(palindromes, seq, &matrix, &complement)
        )),
        "csv" => Ok(format::fmt_csv(palindromes, seq, &matrix, &complement)),
        "custom" => Ok(format::fmt_custom(palindromes, seq)),
        // Already tested in Config::verify but not for a manual Config::new
        other => Err(anyhow!(
            "The given output format: '{}' doesn't exist.",
            other
        )),
    }
}
