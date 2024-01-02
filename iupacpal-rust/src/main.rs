#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
// #![allow(unused_variables)]
// #![allow(unused_imports)]

use libdivsufsort_rs::*;
use std::collections::BTreeSet; // Ordered
use std::fs::File;
use std::io::Write;
use std::time::Instant;
mod config;
mod debug;
mod format;
mod matrix;
mod utils;

use anyhow::Result;
use config::Config;
use debug::print_array;
use format::fmt;
use utils::add_palindromes;
use utils::lcp_array;
use utils::rmq_preprocess;

const DEBUG: bool = false;

pub const IUPAC_SYMBOLS: &str = "acgturyswkmbdhvn*-$#";
pub const IUPAC_SYMBOLS_COUNT: usize = 20;
pub type Palindrome = (i32, i32, i32);
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

fn main() -> Result<()> {
    let start_time = Instant::now();

    // Config and init variables
    let config = Config::from_args();
    let string = config.extract_string()?;
    let seq = string.as_bytes().to_vec();
    let n = seq.len();
    config.verify(n)?;
    
    print!("{}", config.display());
    // dbg!(&string);
    assert!(string.chars().all(|c| IUPAC_SYMBOLS.contains(c)), "Not all chars are in IUPAC");

    // Build matchmatrix
    let matrix = matrix::MatchMatrix::new();
    let complement = build_complement_array();
    // Optionally print match matrix
    // println!("{}", matrix.display(&complement)); 

    // Construct S = seq + '$' + complement(reverse(seq)) + '#'
    let S_n = 2 * n + 2;
    let mut S = vec![0u8; 2 * n + 2];
    for i in 0..n {
        S[i] = seq[i];
        S[n + 1 + i] = complement[seq[n - 1 - i] as usize] as u8;
    }
    S[n] = b'$';
    S[2 * n + 1] = b'#';

    // Construct Suffix Array (SA) & Inverse Suffix Array
    let SA: Vec<INT> = divsufsort64(&S).unwrap();
    let mut invSA: Vec<INT> = vec![0; S_n];
    for i in 0..S_n {
        if let Some(value) = SA.get(i).copied() {
            invSA[value as usize] = i as INT;
        }
    }

    // Calculate LCP & RMQ
    let LCP: Vec<INT> = lcp_array(&S, S_n as INT, &SA, &invSA);
    let A: Vec<INT> = rmq_preprocess(&LCP, S_n as INT);

    if DEBUG {
        print_array("  seq", &seq, n, false);
        print_array("    S", &S, S_n, true);
        print_array("   SA", &SA, S_n, true);
        print_array("invSA", &invSA, S_n, true);
        print_array("  LCP", &LCP, S_n, true);
        print_array("    A", &A, S_n, true);
    }

    // Calculate palidromes
    // TODO: fix types
    let palindromes: BTreeSet<(i32, i32, i32)> = add_palindromes(
        &S,
        S_n as INT,
        n as INT,
        &invSA,
        &LCP,
        &A,
        config.min_len,
        config.max_len,
        config.mismatches,
        config.max_gap,
        &matrix,
    );

    // Print palindromes
    println!("Found n={} palindromes", palindromes.len());
    let palindromes_out = fmt(&config, &palindromes, &seq, n, &matrix, &complement);
    let mut file = File::create(&config.output_file)?;
    writeln!(&mut file, "{}", palindromes_out)?;
    println!("Search complete!");

    let elapsed = start_time.elapsed();
    let elapsed_ms = elapsed.as_millis();
    println!("Elapsed time: {} milliseconds", elapsed_ms);

    Ok(())
}
