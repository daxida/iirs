#![feature(test)]

extern crate elapsed_time;
extern crate test;

mod algo;
mod config;
mod constants;
mod format;
mod matrix;
mod rmq;

use lib::rmq::rmq_preprocess;

use anyhow::Result;
use libdivsufsort_rs::*;
use std::collections::BTreeSet; // Ordered
use std::fs::File;
use std::io::Write;

use algo::{add_palindromes, lcp_array};
use config::Config;
use format::strinfigy_palindromes;

// TODO: fix bug -g 0

fn build_long_sequence(seq: &[u8], n: usize, complement: &[u8; 128]) -> Vec<u8> {
    let mut s = vec![0u8; 2 * n + 2];

    for i in 0..n {
        s[i] = seq[i];
        s[n + 1 + i] = complement[seq[n - 1 - i] as usize] as u8;
        // This should probably be tested also in --release
        assert!(constants::IUPAC_SYMBOLS.contains(seq[i] as char))
    }
    s[n] = b'$';
    s[2 * n + 1] = b'#';

    s
}

// #[elapsed_time::elapsed]
fn find_palindromes(config: &Config, seq: &[u8], n: usize) -> BTreeSet<(i32, i32, i32)> {
    // Build matchmatrix
    let matrix = matrix::MatchMatrix::new();
    let complement = constants::build_complement_array();

    // Construct s = seq + '$' + complement(reverse(seq)) + '#'
    let s_n = 2 * n + 2;
    let s = build_long_sequence(&seq, n, &complement);

    // Construct Suffix Array (sa) & Inverse Suffix Array
    let sa: Vec<i64> = divsufsort64(&s).unwrap();
    let mut inv_sa = vec![0; s_n];
    for (i, value) in sa.iter().enumerate() {
        inv_sa[*value as usize] = i;
    }

    // Calculate LCP & RMQ
    let lcp = lcp_array(&s, s_n, &sa, &inv_sa);
    let rmq_prep = rmq_preprocess(&lcp, s_n); // A in the original

    // Calculate palidromes
    add_palindromes(
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

#[elapsed_time::elapsed]
fn main() -> Result<()> {
    // Config and init variables
    let config = Config::from_args();
    let string = config.extract_string()?;
    let n = string.len();
    config.verify(n)?;
    let seq = string.as_bytes();

    // Find all palindromes
    let palindromes = find_palindromes(&config, &seq, n);

    // Stringify palindromes
    let out_str = strinfigy_palindromes(&config, &palindromes, &seq, n)?;

    // Write palindromes
    let mut file = File::create(&config.output_file)?;
    writeln!(&mut file, "{}", out_str)?;

    println!("\n{}", config.display());
    println!("Search complete!");
    println!("Found n={} palindromes", palindromes.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{config::Config, find_palindromes};

    fn test_seq(config: &Config, string: &str) -> usize {
        let seq = string.to_ascii_lowercase().as_bytes().to_vec();
        let n = seq.len();
        let _ = config.verify(n).unwrap();
        let palindromes = find_palindromes(&config, &seq, n);
        palindromes.len()
    }

    #[test]
    fn test_palindromes() {
        let config = Config::dummy(10, 100, 5, 1);
        let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
        assert_eq!(test_seq(&config, string), 21)
    }

    #[test]
    fn test_palindromes_default_config() {
        let config = Config::dummy_default();
        let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA".repeat(100);
        assert_eq!(test_seq(&config, &string), 10068)
    }

    #[test]
    fn test_palindromes_full_n() {
        let config = Config::dummy(10, 100, 5, 1);
        let string = "N".repeat(500);
        assert_eq!(test_seq(&config, &string), 961)
    }

    #[test]
    fn test_palindromes_full_n_default_config() {
        let config = Config::dummy_default();
        let string = "N".repeat(500);
        assert_eq!(test_seq(&config, &string), 961)
    }

    use test::Bencher;

    #[bench]
    fn bench_palindromes_full_n_default_config(b: &mut Bencher) {
        let config = Config::dummy_default();
        let string = "N".repeat(50000);
        let seq = string.to_ascii_lowercase().as_bytes().to_vec();
        let n = seq.len();
        let _ = config.verify(n).unwrap();
        b.iter(|| find_palindromes(&config, &seq, n))
    }

    use std::collections::BTreeSet;

    // Bemch everything in test_data

    // Option 1

    fn find_palindromes_from_pathconfig(path: &str, config: &Config) -> BTreeSet<(i32, i32, i32)> {
        let string = Config::extract_first_string(String::from(path)).unwrap();
        let seq = string.to_ascii_lowercase().as_bytes().to_vec();
        let n = seq.len();
        config.verify(n).unwrap();
        find_palindromes(&config, &seq, n)
    }

    // Copy pasta a LOT of these
    #[bench]
    fn bench_test1(b: &mut Bencher) {
        let config = Config::dummy(10, 100, 10, 0);
        let path = "test_data/test1.fasta";
        b.iter(|| find_palindromes_from_pathconfig(path, &config))
    }

    #[bench]
    fn bench_alys(b: &mut Bencher) {
        let config = Config::dummy(3, 100, 20, 0);
        let path = "test_data/alys.fna";
        b.iter(|| find_palindromes_from_pathconfig(path, &config))
    }

    // Option 2: make some sort of macro and create benches while iterating test_data
    // (atm it benches everything in bulk)

    // use std::fs;

    // #[bench]
    // fn iterate_test_suite(b: &mut Bencher) {
    //     let folder_path = "test_data";
    //     let config = Config::dummy(10, 100, 10, 1);
    //     let entries = fs::read_dir(folder_path).unwrap();

    //     for entry in entries {
    //         if let Ok(entry) = entry {
    //             if let Some(file_path_str) = entry.path().to_str() {
    //                 // println!("File name: {}", file_name);
    //                 let string = Config::extract_first_string(String::from(file_path_str)).unwrap();
    //                 let seq = string.to_ascii_lowercase().as_bytes().to_vec();
    //                 let n = seq.len();
    //                 config.verify(n).unwrap();
    //                 b.iter(|| find_palindromes(&config, &seq, n))
    //             }
    //         }
    //     }
    // }
}
