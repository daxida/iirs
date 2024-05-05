use super::config::{Config, SearchParams};
use super::constants;
use super::find_palindromes;
use super::matrix;
use super::stringify_palindromes;

// Test for an edge case with truncation (needs complement and matrix).
fn correct_truncation_helper(config: &Config) {
    let string = Config::extract_first_string(&config.output_file).unwrap();
    let seq = string.to_ascii_lowercase().as_bytes().to_vec();
    let n = seq.len();
    config.verify(n).unwrap();
    let palindromes = find_palindromes(&config.params, &seq).unwrap();

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

    for (left, right, _) in palindromes {
        assert!(matrix.match_u8(s[left], complement[s[right] as usize]),);
    }
}

#[test]
fn test_correct_truncation_one() {
    let config = Config {
        params: SearchParams::new(8, 100, 10, 6),
        output_file: String::from("tests/test_data/test1.fasta"),
        ..Default::default()
    };
    correct_truncation_helper(&config)
}

#[test]
fn test_correct_truncation_two() {
    let config = Config {
        params: SearchParams::new(8, 100, 10, 6),
        output_file: String::from("tests/test_data/truncation_edge_case.fasta"),
        ..Default::default()
    };
    correct_truncation_helper(&config)
}

#[test]
fn test_correct_truncation_three() {
    let config = Config {
        params: SearchParams::new(6, 100, 0, 5),
        output_file: String::from("tests/test_data/truncation_edge_case.fasta"),
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
    assert!(stringify_palindromes(&config, &Vec::new(), &Vec::new()).is_err());
}
