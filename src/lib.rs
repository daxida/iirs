#![feature(test)]

extern crate test;

pub mod config;
pub mod format;

mod algo;
mod constants;
mod matrix;
mod rmq;

use config::Config;
use libdivsufsort_rs::*;

/// Panics if the given seq has a character NOT in lowercase IUPAC = "acgturyswkmbdhvn*-"
///
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
        assert!(
            constants::IUPAC_SYMBOLS.contains(seq[i] as char),
            "Error: sequence contains '{}' which is not a IUPAC symbol.",
            seq[i] as char
        )
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
    let rmq_prep = rmq::rmq_preprocess(&lcp, s_n);

    // Calculate palidromes
    algo::add_palindromes(
        &s,
        s_n,
        n,
        &inv_sa,
        &lcp,
        &rmq_prep,
        config.min_len,
        config.max_len,
        config.mismatches,
        config.max_gap,
        &matrix,
    )
}

#[cfg(test)]
mod tests {
    use crate::{config::Config, find_palindromes};
    use test::Bencher;

    fn test_seq(config: &Config, string: &str) -> usize {
        let seq = string.to_ascii_lowercase().as_bytes().to_vec();
        let n = seq.len();
        let _ = config.verify(n).unwrap();
        let palindromes = find_palindromes(&config, &seq);
        palindromes.len()
    }

    #[test]
    fn test_palindromes_default_config() {
        let config = Config::dummy_default();
        let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA".repeat(100);
        assert_eq!(test_seq(&config, &string), 10068)
    }

    #[test]
    fn test_palindromes_custom_config() {
        let config = Config::dummy(10, 100, 5, 1);
        let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
        assert_eq!(test_seq(&config, string), 21)
    }

    #[test]
    fn test_palindromes_no_mismatches() {
        let config = Config::dummy(10, 100, 5, 0);
        let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
        assert_eq!(test_seq(&config, string), 14)
    }

    #[test]
    fn test_palindromes_full_n_default_config() {
        let config = Config::dummy_default();
        let string = "N".repeat(500);
        assert_eq!(test_seq(&config, &string), 961)
    }

    #[test]
    fn test_palindromes_full_n_custom_config() {
        let config = Config::dummy(10, 100, 5, 1);
        let string = "N".repeat(500);
        assert_eq!(test_seq(&config, &string), 961)
    }

    #[test]
    fn test_palindromes_full_n_no_gap() {
        let config = Config::dummy(10, 100, 0, 1);
        let string = "N".repeat(500);
        assert_eq!(test_seq(&config, &string), 481)
    }

    // Start test from local files
    //
    // Test generator
    fn find_palindromes_from_pathconfig(path: &str, config: &Config) -> Vec<(i32, i32, i32)> {
        let string = Config::extract_first_string(String::from(path)).unwrap();
        let seq = string.to_ascii_lowercase().as_bytes().to_vec();
        let n = seq.len();
        config.verify(n).unwrap();
        find_palindromes(&config, &seq)
    }

    #[test]
    fn test_palindromes_alys() {
        let config = Config::dummy(3, 100, 20, 0);
        let path = "test_data/alys.fna";
        assert_eq!(
            find_palindromes_from_pathconfig(&path, &config).len(),
            739728
        )
    }

    #[test]
    fn test_palindromes_8100_n() {
        let config = Config::dummy(3, 100, 20, 0);
        let path = "test_data/8100N.fasta";
        assert_eq!(
            find_palindromes_from_pathconfig(&path, &config).len(),
            16189
        )
    }
    #[test]
    fn test_palindromes_d00596() {
        let config = Config::dummy(3, 100, 20, 0);
        let path = "test_data/d00596.fasta";
        assert_eq!(
            find_palindromes_from_pathconfig(&path, &config).len(),
            5251
        )
    }

    // Benchmark

    #[bench]
    fn bench_palindromes_full_n_default_config(b: &mut Bencher) {
        let config = Config::dummy_default();
        let string = "N".repeat(50000);
        let seq = string.to_ascii_lowercase().as_bytes().to_vec();
        let n = seq.len();
        let _ = config.verify(n).unwrap();
        b.iter(|| find_palindromes(&config, &seq))
    }

    #[bench]
    fn bench_test1(b: &mut Bencher) {
        let config = Config::dummy(10, 100, 10, 0);
        let path = "test_data/test1.fasta";
        b.iter(|| find_palindromes_from_pathconfig(path, &config))
    }

    #[bench]
    fn bench_default_rand_iupac_1000(b: &mut Bencher) {
        let config = Config::dummy_default();
        let path = "test_data/randIUPAC1000.fasta";
        b.iter(|| find_palindromes_from_pathconfig(path, &config))
    }

    #[bench]
    fn bench_default_rand_iupac_10000(b: &mut Bencher) {
        let config = Config::dummy_default();
        let path = "test_data/randIUPAC10000.fasta";
        b.iter(|| find_palindromes_from_pathconfig(path, &config))
    }

    // #[bench]
    // fn bench_default_rand_iupac_100000(b: &mut Bencher) {
    //     let config = Config::dummy_default();
    //     let path = "test_data/randIUPAC100000.fasta";
    //     b.iter(|| find_palindromes_from_pathconfig(path, &config))
    // }

    #[bench]
    fn bench_alys(b: &mut Bencher) {
        let config = Config::dummy(3, 100, 20, 0);
        let path = "test_data/alys.fna";
        b.iter(|| find_palindromes_from_pathconfig(path, &config))
    }

    // #[bench]
    // fn bench_default_rand_iupac_1000000(b: &mut Bencher) {
    //     let config = Config::dummy_default();
    //     let path = "test_data/randIUPAC1000000.fasta";
    //     b.iter(|| find_palindromes_from_pathconfig(path, &config))
    // }
}
