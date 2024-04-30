use super::config::Config;
use super::constants;
use super::find_palindromes;
use super::matrix;

// Test for an edge case with truncation (needs complement and matrix).
fn correct_truncation_helper(config: &Config, path: &str) {
    let string = Config::extract_first_string(String::from(path)).unwrap();
    let seq = string.to_ascii_lowercase().as_bytes().to_vec();
    let n = seq.len();
    config.verify(n).unwrap();
    let palindromes = find_palindromes(&config, &seq).unwrap();

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
    let config = Config::dummy(8, 100, 10, 6);
    let path = "tests/test_data/test1.fasta";
    correct_truncation_helper(&config, path)
}

#[test]
fn test_correct_truncation_two() {
    let config = Config::dummy(8, 100, 10, 6);
    let path = "tests/test_data/truncation_edge_case.fasta";
    correct_truncation_helper(&config, path)
}

#[test]
fn test_correct_truncation_three() {
    let config = Config::dummy(6, 100, 0, 5);
    let path = "tests/test_data/truncation_edge_case.fasta";
    correct_truncation_helper(&config, path)
}
