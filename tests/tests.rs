use iupacpal::{
    config::{Config, Parameters},
    find_palindromes,
};

fn test_seq(parameters: &Parameters, string: &str) -> usize {
    let seq = string.to_ascii_lowercase().as_bytes().to_vec();
    parameters.verify_bounds(seq.len()).unwrap();
    let palindromes = find_palindromes(&parameters, &seq).unwrap();
    palindromes.len()
}

#[test]
fn test_palindromes_default_config() {
    let config = Parameters::default();
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA".repeat(100);
    assert_eq!(test_seq(&config, &string), 10068)
}

#[test]
fn test_palindromes_custom_config() {
    let config = Parameters::new(10, 100, 5, 1);
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
    assert_eq!(test_seq(&config, string), 21)
}

#[test]
fn test_palindromes_no_mismatches() {
    let config = Parameters::new(10, 100, 5, 0);
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
    assert_eq!(test_seq(&config, string), 14)
}

#[test]
fn test_palindromes_no_gap_with_mismatches() {
    let config = Parameters::new(10, 100, 0, 5);
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
    assert_eq!(test_seq(&config, string), 17)
}

#[test]
fn test_palindromes_no_mismatches_min_len_two() {
    let config = Parameters::new(2, 100, 5, 0);
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
    assert_eq!(test_seq(&config, string), 58)
}

#[test]
fn test_palindromes_no_mismatches_min_len_two_no_gap() {
    let config = Parameters::new(2, 100, 0, 0);
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
    assert_eq!(test_seq(&config, string), 18)
}

#[test]
fn test_palindromes_full_n_default_config() {
    let config = Parameters::default();
    let string = "N".repeat(500);
    assert_eq!(test_seq(&config, &string), 961)
}

#[test]
fn test_palindromes_full_n_custom_config() {
    let config = Parameters::new(10, 100, 5, 1);
    let string = "N".repeat(500);
    assert_eq!(test_seq(&config, &string), 961)
}

#[test]
fn test_palindromes_full_n_no_gap() {
    let config = Parameters::new(10, 100, 0, 1);
    let string = "N".repeat(500);
    assert_eq!(test_seq(&config, &string), 481)
}

#[test]
fn test_palindromes_full_n_5000() {
    let config = Parameters::default();
    let string = "N".repeat(50000);
    assert_eq!(test_seq(&config, &string), 99961);
}

// Start test from local files
//
// Test generator
fn find_palindromes_from_pathconfig(config: &Config) -> Vec<(usize, usize, usize)> {
    let string = Config::extract_first_string(&config.output_file).unwrap();
    let seq = string.to_ascii_lowercase().as_bytes().to_vec();
    config.verify(seq.len()).unwrap();
    find_palindromes(&config.parameters, &seq).unwrap()
}

#[test]
fn test_palindromes_edge_gap() {
    // The original IUPACpal won't find this palindrome
    let output_file = String::from("tests/test_data/edge_gap.fasta");
    let parameters = Parameters::new(14, 100, 3, 0);
    let config = Config {
        output_file,
        parameters,
        ..Default::default()
    };
    assert_eq!(find_palindromes_from_pathconfig(&config).len(), 1)
}

#[test]
fn test_palindromes_alys() {
    let output_file = String::from("tests/test_data/alys.fna");
    let parameters = Parameters::new(3, 100, 20, 0);
    let config = Config {
        output_file,
        parameters,
        ..Default::default()
    };
    assert_eq!(find_palindromes_from_pathconfig(&config).len(), 739728)
}

#[test]
fn test_palindromes_8100_n() {
    let output_file = String::from("tests/test_data/8100N.fasta");
    let parameters = Parameters::new(3, 100, 20, 0);
    let config = Config {
        output_file,
        parameters,
        ..Default::default()
    };
    assert_eq!(find_palindromes_from_pathconfig(&config).len(), 16189)
}

#[test]
fn test_palindromes_8100_n_with_mismatches() {
    let output_file = String::from("tests/test_data/8100N.fasta");
    let parameters = Parameters::new(3, 100, 20, 2);
    let config = Config {
        output_file,
        parameters,
        ..Default::default()
    };
    assert_eq!(find_palindromes_from_pathconfig(&config).len(), 16189)
}

#[test]
fn test_palindromes_d00596() {
    let output_file = String::from("tests/test_data/d00596.fasta");
    let parameters = Parameters::new(3, 100, 20, 0);
    let config = Config {
        output_file,
        parameters,
        ..Default::default()
    };
    assert_eq!(find_palindromes_from_pathconfig(&config).len(), 5251)
}

#[test]
fn test_palindromes_d00596_with_mismatches() {
    let output_file = String::from("tests/test_data/d00596.fasta");
    let parameters = Parameters::new(3, 100, 20, 2);
    let config = Config {
        output_file,
        parameters,
        ..Default::default()
    };
    assert_eq!(find_palindromes_from_pathconfig(&config).len(), 31555)
}

#[test]
fn test_rand_1000() {
    let output_file = String::from("tests/test_data/rand1000.fasta");
    let parameters = Parameters::new(3, 100, 20, 0);
    let config = Config {
        output_file,
        parameters,
        ..Default::default()
    };
    assert_eq!(find_palindromes_from_pathconfig(&config).len(), 254)
}

#[test]
fn test_rand_10000() {
    let output_file = String::from("tests/test_data/rand10000.fasta");
    let parameters = Parameters::new(3, 100, 20, 0);
    let config = Config {
        output_file,
        parameters,
        ..Default::default()
    };
    assert_eq!(find_palindromes_from_pathconfig(&config).len(), 2484)
}

#[test]
fn test_rand_100000() {
    let output_file = String::from("tests/test_data/rand100000.fasta");
    let parameters = Parameters::new(3, 100, 20, 0);
    let config = Config {
        output_file,
        parameters,
        ..Default::default()
    };
    assert_eq!(find_palindromes_from_pathconfig(&config).len(), 25440)
}

#[test]
fn test_rand_1000000() {
    let output_file = String::from("tests/test_data/rand1000000.fasta");
    let parameters = Parameters::new(3, 100, 20, 0);
    let config = Config {
        output_file,
        parameters,
        ..Default::default()
    };
    assert_eq!(find_palindromes_from_pathconfig(&config).len(), 253566)
}

#[test]
fn test_test_1() {
    let output_file = String::from("tests/test_data/test1.fasta");
    let parameters = Parameters::new(3, 100, 20, 0);
    let config = Config {
        output_file,
        parameters,
        ..Default::default()
    };
    assert_eq!(find_palindromes_from_pathconfig(&config).len(), 84)
}
