mod cli;
pub use cli::Cli;

mod config;
pub use config::{Config, SearchParams};

mod algo;
mod constants;
mod format;
mod matrix;
mod utils;

use anyhow::Result;

/// Find all the [Inverted Repeats](https://en.wikipedia.org/wiki/Inverted_repeat) (IRs) in a sequence
/// based on the provided parameters.
///
/// Each IR is returned a tuple of three integers (usize): start position, end position, and gap size.
///
/// # Examples
///
/// ```rust
/// use iirs::{SearchParams, find_irs};
///
/// let seq = "acbbgt".as_bytes();
/// let params = SearchParams::new(3, 6, 2, 0).unwrap();
/// assert!(params.check_bounds(seq.len()).is_ok());
/// let irs = find_irs(&params, &seq);
/// // The only IR in the sequence is "acbbg"
/// assert_eq!(irs.unwrap(), vec![(0, 5, 0)]);
///
/// // Returns an error if the given sequence contains invalid characters
/// let seq = "jj".as_bytes();
/// let irs = find_irs(&params, &seq);
/// assert!(irs.is_err());
///
/// // It is not case-sensitive and ignores newlines.
/// let seq = "ACB\n\rBGT".as_bytes();
/// let irs = find_irs(&params, &seq);
/// assert_eq!(irs.unwrap(), vec![(0, 5, 0)]);
/// ```
#[elapsed_time::elapsed]
pub fn find_irs(params: &SearchParams, seq: &[u8]) -> Result<Vec<(usize, usize, usize)>> {
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
    // By default use the Sparse Table implementation for the Rmq
    #[cfg(not(feature = "tabulation"))]
    let rmq = rmq::Sparse::new(&lcp);
    #[cfg(feature = "tabulation")]
    let rmq = rmq::Tabulation::new(&lcp);

    // Calculate inverted repeats
    let mut irs = algo::add_irs(&s, &inv_sa, &rmq, params, &matrix);

    // Deal with the sorting strategy.
    // Alternatives, or even skipping sorting altogether, can improve the performance.
    // The original IUPACpal sorts by (left, gap_size, -right)
    irs.sort_by(|a, b| {
        let cmp_left = a.0.cmp(&b.0);
        let cmp_gap = a.2.cmp(&a.2);
        let cmp_right = b.1.cmp(&a.1);
        cmp_left.then(cmp_gap).then(cmp_right)
    });

    Ok(irs)
}

/// Stringify the given [Inverted Repeats](https://en.wikipedia.org/wiki/Inverted_repeat) (IRs)
/// based on the specified output format in the configuration.
///
/// An error is returned for an invalid output format.
/// Valid formats are: classic (same as IUPACpal), csv and custom.
///
/// If the specified format is valid, the function returns a tuple of Strings: (header, stringified_irs).
///
/// # Examples
///
/// ```rust
/// use iirs::{SearchParams, Config};
/// use iirs::{find_irs, stringify_irs};
///
/// // Simple example for the csv output format.
/// let seq = "acbbgt".as_bytes();
/// let config = Config {
///     params: SearchParams::new(3, 6, 2, 0).unwrap(),
///     output_format: "csv",
///     // The remaining fields are not relevant here.
///     ..Default::default()
/// };
/// let irs = find_irs(&config.params, &seq).unwrap();
/// let (header, irs_str) = stringify_irs(&config, &irs, &seq, 0).unwrap();
/// let expected = "\
///     start_n,end_n,nucleotide,start_ir,end_ir,reverse_complement,matching\n\
///     1,3,acb,6,4,tgb,111\n";
/// assert_eq!(format!("{}\n{}", &header, &irs_str), expected);
///
/// // For the classic method, all the fields are used in the header.
/// let config = Config::new("in.fasta", "seq0", 3, 6, 2, 0, "out.txt", "classic").unwrap();
/// let (header, irs_str) = stringify_irs(&config, &irs, &seq, 0).unwrap();
/// let expected = "\
///     Palindromes of: in.fasta\n\
///     Sequence name: seq0\n\
///     Sequence length is: 6\n\
///     Start at position: 1\n\
///     End at position: 6\n\
///     Minimum length of Palindromes is: 3\n\
///     Maximum length of Palindromes is: 6\n\
///     Maximum gap between elements is: 2\n\
///     Number of mismatches allowed in Palindrome: 0\n\n\n\n\
///     Palindromes:";
/// assert_eq!(header, expected);
/// ```
pub fn stringify_irs(
    config: &Config,
    irs: &Vec<(usize, usize, usize)>,
    seq: &[u8],
    offset: usize,
) -> Result<(String, String)> {
    let matrix = matrix::MatchMatrix::new();
    let complement = constants::build_complement_array();

    utils::verify_format(config.output_format)?;

    match config.output_format {
        "classic" => Ok((
            format::fmt_classic_header(config, seq.len()),
            format::fmt_classic(irs, seq, &matrix, &complement),
        )),
        "csv" => Ok((
            format::fmt_csv_header(),
            format::fmt_csv(irs, seq, &matrix, &complement),
        )),
        "custom" => Ok((format::fmt_custom_header(), format::fmt_custom(irs, seq, offset))),
        // Already tested in utils::verify_format
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod libtests;
