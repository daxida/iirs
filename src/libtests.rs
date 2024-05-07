use anyhow::Result;
use seq_io::fasta::{Reader, Record};

use super::config::{Config, SearchParams};
use super::constants;
use super::find_irs;
use super::matrix;
use super::stringify_irs;
use super::utils;

/// Attemps to extract the first sequence (string) from the fasta file. Returns a trimmed lowercase String.
///
/// Returns an error if there are no sequences.
fn extract_first_sequence(config: &Config) -> Result<String> {
    utils::check_file_exist(&config.input_file)?;
    let mut reader = Reader::from_path(&config.input_file)?;
    let record = reader
        .next()
        .expect("No sequences found")
        .expect("Error reading record");

    Ok(std::str::from_utf8(record.seq())
        .unwrap()
        .to_lowercase()
        .replace('\n', ""))
}

// Test for an edge case with truncation (needs complement and matrix).
fn correct_truncation_helper(config: &Config) {
    let string = extract_first_sequence(config).unwrap();
    let seq = string.to_ascii_lowercase().as_bytes().to_vec();
    let n = seq.len();
    config.verify(n).unwrap();
    let irs = find_irs(&config.params, &seq).unwrap();

    let complement = constants::build_complement_array();
    let s_n = 2 * n + 2;
    let mut s = vec![0u8; s_n];
    for i in 0..n {
        s[i] = seq[i];
        s[n + 1 + i] = complement[seq[n - 1 - i] as usize] as u8;
    }
    s[n] = b'$';
    s[2 * n + 1] = b'#';
    let matrix = matrix::MatchMatrix::new();

    for (left, right, _) in irs {
        assert!(matrix.match_u8(s[left], complement[s[right] as usize]),);
    }
}

#[test]
fn test_correct_truncation_one() {
    let config = Config {
        params: SearchParams::new(8, 100, 10, 6),
        input_file: String::from("tests/test_data/test1.fasta"),
        ..Default::default()
    };
    correct_truncation_helper(&config)
}

#[test]
fn test_correct_truncation_two() {
    let config = Config {
        params: SearchParams::new(8, 100, 10, 6),
        input_file: String::from("tests/test_data/truncation_edge_case.fasta"),
        ..Default::default()
    };
    correct_truncation_helper(&config)
}

#[test]
fn test_correct_truncation_three() {
    let config = Config {
        params: SearchParams::new(6, 100, 0, 5),
        input_file: String::from("tests/test_data/truncation_edge_case.fasta"),
        ..Default::default()
    };
    correct_truncation_helper(&config)
}

#[test]
fn test_invalid_output_format_stringify() {
    let config = Config {
        output_format: String::from("inexistent"),
        ..Default::default()
    };
    assert!(stringify_irs(&config, &Vec::new(), &Vec::new()).is_err());
}

// Tests from local files
//
// Test generator
fn find_irs_from_first_sequence(config: &Config) -> Vec<(usize, usize, usize)> {
    let string = extract_first_sequence(config).unwrap();
    let seq = string.to_ascii_lowercase().as_bytes().to_vec();
    config.verify(seq.len()).unwrap();
    find_irs(&config.params, &seq).unwrap()
}

#[test]
fn test_irs_edge_gap() {
    // The original IUPACpal won't find this IR
    let config = Config {
        params: SearchParams::new(14, 100, 3, 0),
        input_file: String::from("tests/test_data/edge_gap.fasta"),
        ..Default::default()
    };
    assert_eq!(find_irs_from_first_sequence(&config).len(), 1)
}

#[test]
fn test_irs_alys() {
    let config = Config {
        params: SearchParams::new(3, 100, 20, 0),
        input_file: String::from("tests/test_data/alys.fna"),
        ..Default::default()
    };
    assert_eq!(find_irs_from_first_sequence(&config).len(), 739728)
}

#[test]
fn test_irs_8100_n() {
    let config = Config {
        params: SearchParams::new(3, 100, 20, 0),
        input_file: String::from("tests/test_data/8100N.fasta"),
        ..Default::default()
    };
    assert_eq!(find_irs_from_first_sequence(&config).len(), 16189)
}

#[test]
fn test_irs_8100_n_with_mismatches() {
    let config = Config {
        params: SearchParams::new(3, 100, 20, 2),
        input_file: String::from("tests/test_data/8100N.fasta"),
        ..Default::default()
    };
    assert_eq!(find_irs_from_first_sequence(&config).len(), 16189)
}

#[test]
fn test_irs_d00596() {
    let config = Config {
        params: SearchParams::new(3, 100, 20, 0),
        input_file: String::from("tests/test_data/d00596.fasta"),
        ..Default::default()
    };
    assert_eq!(find_irs_from_first_sequence(&config).len(), 5251)
}

#[test]
fn test_irs_d00596_with_mismatches() {
    let config = Config {
        params: SearchParams::new(3, 100, 20, 2),
        input_file: String::from("tests/test_data/d00596.fasta"),
        ..Default::default()
    };
    assert_eq!(find_irs_from_first_sequence(&config).len(), 31555)
}

#[test]
fn test_rand_1000() {
    let config = Config {
        params: SearchParams::new(3, 100, 20, 0),
        input_file: String::from("tests/test_data/rand1000.fasta"),
        ..Default::default()
    };
    assert_eq!(find_irs_from_first_sequence(&config).len(), 254)
}

#[test]
fn test_rand_10000() {
    let config = Config {
        params: SearchParams::new(3, 100, 20, 0),
        input_file: String::from("tests/test_data/rand10000.fasta"),
        ..Default::default()
    };
    assert_eq!(find_irs_from_first_sequence(&config).len(), 2484)
}

#[test]
fn test_rand_100000() {
    let config = Config {
        params: SearchParams::new(3, 100, 20, 0),
        input_file: String::from("tests/test_data/rand100000.fasta"),
        ..Default::default()
    };
    assert_eq!(find_irs_from_first_sequence(&config).len(), 25440)
}

#[test]
fn test_rand_1000000() {
    let config = Config {
        params: SearchParams::new(3, 100, 20, 0),
        input_file: String::from("tests/test_data/rand1000000.fasta"),
        ..Default::default()
    };
    assert_eq!(find_irs_from_first_sequence(&config).len(), 253566)
}

#[test]
fn test_test_1() {
    let config = Config {
        params: SearchParams::new(3, 100, 20, 0),
        input_file: String::from("tests/test_data/test1.fasta"),
        ..Default::default()
    };
    assert_eq!(find_irs_from_first_sequence(&config).len(), 84)
}
