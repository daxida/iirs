use iupacpal::{config::SearchParams, find_palindromes};

fn test_seq(params: &SearchParams, string: &str) -> usize {
    let seq = string.to_ascii_lowercase().as_bytes().to_vec();
    params.verify_bounds(seq.len()).unwrap();
    let palindromes = find_palindromes(&params, &seq).unwrap();
    palindromes.len()
}

#[test]
fn test_palindromes_default_params() {
    let params = SearchParams::default();
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA".repeat(100);
    assert_eq!(test_seq(&params, &string), 10068)
}

#[test]
fn test_palindromes_custom_params() {
    let params = SearchParams::new(10, 100, 5, 1);
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
    assert_eq!(test_seq(&params, string), 21)
}

#[test]
fn test_palindromes_no_mismatches() {
    let params = SearchParams::new(10, 100, 5, 0);
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
    assert_eq!(test_seq(&params, string), 14)
}

#[test]
fn test_palindromes_no_gap_with_mismatches() {
    let params = SearchParams::new(10, 100, 0, 5);
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
    assert_eq!(test_seq(&params, string), 17)
}

#[test]
fn test_palindromes_no_mismatches_min_len_two() {
    let params = SearchParams::new(2, 100, 5, 0);
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
    assert_eq!(test_seq(&params, string), 58)
}

#[test]
fn test_palindromes_no_mismatches_min_len_two_no_gap() {
    let params = SearchParams::new(2, 100, 0, 0);
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
    assert_eq!(test_seq(&params, string), 18)
}

#[test]
fn test_palindromes_full_n_default_params() {
    let params = SearchParams::default();
    let string = "N".repeat(500);
    assert_eq!(test_seq(&params, &string), 961)
}

#[test]
fn test_palindromes_full_n_custom_params() {
    let params = SearchParams::new(10, 100, 5, 1);
    let string = "N".repeat(500);
    assert_eq!(test_seq(&params, &string), 961)
}

#[test]
fn test_palindromes_full_n_no_gap() {
    let params = SearchParams::new(10, 100, 0, 1);
    let string = "N".repeat(500);
    assert_eq!(test_seq(&params, &string), 481)
}

#[test]
fn test_palindromes_full_n_5000() {
    let params = SearchParams::default();
    let string = "N".repeat(50000);
    assert_eq!(test_seq(&params, &string), 99961);
}
