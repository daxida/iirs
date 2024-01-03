// #![allow(non_upper_case_globals)]
// #![allow(non_camel_case_types)]
// #![allow(non_snake_case)]
// #![allow(dead_code)]
// #![allow(unused_variables)]
// #![allow(unused_imports)]

mod config;
mod debug;
mod experiments;
mod format;
mod matrix;
mod utils;

use anyhow::{anyhow, Result};
use clap::CommandFactory;
use libdivsufsort_rs::*;
use std::collections::BTreeSet; // Ordered
use std::fs::File;
use std::io::Write;
use std::time::Instant;

use config::Config;
use debug::print_array;
use format::fmt;
use format::fmt_csv;
use matrix::MatchMatrix;
use utils::add_palindromes;
use utils::lcp_array;
use utils::rmq_preprocess;

const DEBUG: bool = false;
const IUPAC_SYMBOLS: &str = "acgturyswkmbdhvn*-";

pub type INT = i64;

fn build_complement_array() -> [char; 128] {
    let complement_rules = vec![
        ('a', 't'),
        ('c', 'g'),
        ('g', 'c'),
        ('t', 'a'),
        ('u', 'a'),
        ('r', 'y'),
        ('y', 'r'),
        ('s', 's'),
        ('w', 'w'),
        ('k', 'm'),
        ('m', 'k'),
        ('b', 'v'),
        ('d', 'h'),
        ('h', 'd'),
        ('v', 'b'),
        ('n', 'n'),
        ('*', 'n'),
        ('-', 'n'),
    ];
    let mut complement: [char; 128] = ['@'; 128];

    for (key, value) in complement_rules {
        complement[key as usize] = value;
    }

    complement
}

fn find_palindromes(
    config: &Config,
    seq: &[u8],
    n: usize,
    complement: &[char; 128],
    matrix: &MatchMatrix,
) -> BTreeSet<(i32, i32, i32)> {
    // Construct s = seq + '$' + complement(reverse(seq)) + '#'
    let s_n = 2 * n + 2;
    let mut s = vec![0u8; 2 * n + 2];
    for i in 0..n {
        s[i] = seq[i];
        s[n + 1 + i] = complement[seq[n - 1 - i] as usize] as u8;
    }
    s[n] = b'$';
    s[2 * n + 1] = b'#';

    // Construct Suffix Array (sa) & Inverse Suffix Array
    let sa: Vec<INT> = divsufsort64(&s).unwrap();
    let mut inv_sa: Vec<INT> = vec![0; s_n];
    for (i, value) in sa.iter().enumerate() {
        inv_sa[*value as usize] = i as INT;
    }

    // Calculate LCP & RMQ
    let lcp: Vec<INT> = lcp_array(&s, s_n, &sa, &inv_sa);
    let ary: Vec<INT> = rmq_preprocess(&lcp, s_n); // A in the original

    if DEBUG {
        print_array("  seq", seq, false);
        print_array("    S", &s, true);
        print_array("   SA", &sa, true);
        print_array("invSA", &inv_sa, true);
        print_array("  LCP", &lcp, true);
        print_array("A/ary", &ary, true);
    }

    // Calculate palidromes
    // TODO: fix types
    let palindromes: BTreeSet<(i32, i32, i32)> = add_palindromes(
        &s,
        s_n as INT,
        n as INT,
        &inv_sa,
        &lcp,
        &ary,
        config.min_len,
        config.max_len,
        config.mismatches,
        config.max_gap,
        matrix,
    );

    palindromes
}

fn main() -> Result<()> {
    let start_time = Instant::now();

    // Config and init variables
    let config = Config::from_args();
    let string = config.extract_string()?;
    let seq = string.as_bytes().to_vec();
    let n = seq.len();
    if let Err(msg) = config.verify(n) {
        let _ = Config::command().print_help();
        println!();
        return Err(msg);
    }
    config.verify(n)?;

    print!("{}", config.display());
    // dbg!(&string);
    assert!(
        string.chars().all(|c| IUPAC_SYMBOLS.contains(c)),
        "Not all chars are in IUPAC"
    );

    // Build matchmatrix
    let matrix = matrix::MatchMatrix::new();
    let complement = build_complement_array();
    // Optionally print match matrix
    // println!("{}", matrix.display(&complement));

    let elapsed = start_time.elapsed();
    let elapsed_ms = elapsed.as_millis();
    println!("Elapsed time: {} milliseconds (PRECOMP)", elapsed_ms);

    // START TEST
    //
    // Construct s = seq + '$' + complement(reverse(seq)) + '#'
    let s_n = 2 * n + 2;
    let mut s = vec![0u8; 2 * n + 2];
    for i in 0..n {
        s[i] = seq[i];
        s[n + 1 + i] = complement[seq[n - 1 - i] as usize] as u8;
    }
    s[n] = b'$';
    s[2 * n + 1] = b'#';

    // Construct Suffix Array (sa) & Inverse Suffix Array
    let sa: Vec<INT> = divsufsort64(&s).unwrap();
    let mut inv_sa: Vec<INT> = vec![0; s_n];
    for (i, value) in sa.iter().enumerate() {
        inv_sa[*value as usize] = i as INT;
    }

    // Calculate LCP & RMQ
    let lcp: Vec<INT> = lcp_array(&s, s_n, &sa, &inv_sa);

    let elapsed = start_time.elapsed();
    let elapsed_ms = elapsed.as_millis();

    println!("Elapsed time: {} milliseconds (LCP)", elapsed_ms);

    let ary: Vec<INT> = rmq_preprocess(&lcp, s_n); // A in the original

    let elapsed = start_time.elapsed();
    let elapsed_ms = elapsed.as_millis();
    println!("Elapsed time: {} milliseconds (RMQ)", elapsed_ms);

    if DEBUG {
        print_array("  seq", &seq, false);
        print_array("    S", &s, true);
        print_array("   SA", &sa, true);
        print_array("invSA", &inv_sa, true);
        print_array("  LCP", &lcp, true);
        print_array("A/ary", &ary, true);
    }

    // Calculate palidromes
    // TODO: fix types
    let palindromes: BTreeSet<(i32, i32, i32)> = add_palindromes(
        &s,
        s_n as INT,
        n as INT,
        &inv_sa,
        &lcp,
        &ary,
        config.min_len,
        config.max_len,
        config.mismatches,
        config.max_gap,
        &matrix,
    );

    //
    // END TEST
    // let palindromes = find_palindromes(&config, &seq, n, &complement, &matrix);

    let elapsed = start_time.elapsed();
    let elapsed_ms = elapsed.as_millis();
    println!(
        "Elapsed time: {} milliseconds (END ADD PALINDROMES)",
        elapsed_ms
    );

    // Print palindromes
    println!("Found n={} palindromes", palindromes.len());
    let out_str = match config.output_format.as_str() {
        "classic" => {
            format!(
                "{}{}",
                Config::out_palindrome_display(&config, n),
                fmt(&palindromes, &seq, &matrix, &complement)
            )
        }
        "csv" => fmt_csv(&palindromes, &seq),
        _ => {
            let _ = Config::command().print_help();
            return Err(anyhow!(
                "Output format '{}' not supported",
                config.output_format
            ));
        }
    };

    let elapsed = start_time.elapsed();
    let elapsed_ms = elapsed.as_millis();
    println!(
        "Elapsed time: {} milliseconds (END GET PALINDROME STRING)",
        elapsed_ms
    );

    // Write palindromes
    let mut file = File::create(&config.output_file)?;
    writeln!(&mut file, "{}", out_str)?;
    println!("Search complete!");

    let elapsed = start_time.elapsed();
    let elapsed_ms = elapsed.as_millis();
    println!("Elapsed time: {} milliseconds (TOTAL)", elapsed_ms);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{build_complement_array, config::Config, find_palindromes, matrix};

    fn test_seq(config: &Config, string: &str) -> usize {
        let seq = string.to_ascii_lowercase().as_bytes().to_vec();
        let n = seq.len();
        let _ = config.verify(n).unwrap();
        let matrix = matrix::MatchMatrix::new();
        let complement = build_complement_array();
        let palindromes = find_palindromes(&config, &seq, n, &complement, &matrix);
        palindromes.len()
    }

    #[test]
    fn test_palindromes() {
        let config = Config::new("f", "f", 10, 100, 5, 1, "f", "f");
        let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
        assert_eq!(test_seq(&config, string), 21)
    }

    #[test]
    fn test_palindromes_full_n() {
        let config = Config::new("f", "f", 10, 100, 5, 1, "f", "f");
        let string = "N".repeat(500);
        let string = string.as_str();
        assert_eq!(test_seq(&config, string), 961)
    }
}
