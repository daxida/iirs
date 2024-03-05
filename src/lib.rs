pub mod config;

mod algo;
mod constants;
mod format;
mod matrix;
mod rmq;

use anyhow::{anyhow, Result};
use config::Config;
use constants::IUPAC_SYMBOLS;
use libdivsufsort_rs::*;

/// Find palindromes in a sequence based on the provided configuration.
///
/// Each palindrome is a tuple of three integers (usize): start position, end position, and gap size.
///
/// # Examples
///
/// ```rust
/// use iupacpal::{config::Config, find_palindromes};
///
/// let seq = "acbbgt".as_bytes();
/// let config = Config::dummy(3, 6, 2, 0);
/// let palindromes = find_palindromes(&config, &seq);
/// assert_eq!(palindromes.unwrap(), vec![(0, 5, 0)]);
///
/// // Returns an error if the given sequence contains invalid characters
/// let seq = "jj".as_bytes();
/// let palindromes = find_palindromes(&config, &seq);
/// assert!(palindromes.is_err());
/// ```
#[elapsed_time::elapsed]
pub fn find_palindromes(config: &Config, seq: &[u8]) -> Result<Vec<(usize, usize, usize)>> {
    // Build matchmatrix
    let matrix = matrix::MatchMatrix::new();
    let complement = constants::build_complement_array();
    
    // Construct s = seq + '$' + complement(reverse(seq)) + '#'
    let n = seq.len();
    let s_n = 2 * n + 2;
    let mut s = vec![0u8; s_n];
    for i in 0..n {
        // Note that this was already checked if we were using the CLI. We recheck it
        // because it's cheap and makes sense in the standalone version of this function
        if !IUPAC_SYMBOLS.contains(seq[i] as char) {
            return Err(anyhow!(
                "sequence contains '{}' which is not an IUPAC symbol.",
                seq[i] as char
            ));
        }
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
        &inv_sa,
        &rmq,
        config.min_len,
        config.max_len,
        config.mismatches,
        config.max_gap,
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

    Ok(palindromes)
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
/// let palindromes = find_palindromes(&config, &seq).unwrap();
/// let out_str = strinfigy_palindromes(&config, &palindromes, &seq).unwrap();
/// let expected = "\
///     start_n,end_n,nucleotide,start_ir,end_ir,reverse_complement,matching\n\
///     1,3,acb,6,4,tgb,111\n";
///
/// assert_eq!(out_str, expected);
/// ```
pub fn strinfigy_palindromes(
    config: &Config,
    palindromes: &Vec<(usize, usize, usize)>,
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
