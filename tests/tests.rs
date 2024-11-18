use iirs::{find_irs, SearchParams};

fn test_amount_irs(params: &SearchParams, string: &str) -> usize {
    let seq = string.to_ascii_lowercase().as_bytes().to_vec();
    params.check_bounds(seq.len()).unwrap();
    find_irs(params, &seq).unwrap().len()
}

#[test]
fn test_irs_default_params() {
    let params = SearchParams::default();
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA".repeat(100);
    assert_eq!(test_amount_irs(&params, &string), 10068)
}

#[test]
fn test_irs_custom_params() {
    let params = SearchParams::new(10, 100, 5, 1).unwrap();
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
    assert_eq!(test_amount_irs(&params, string), 21)
}

#[test]
fn test_irs_no_mismatches() {
    let params = SearchParams::new(10, 100, 5, 0).unwrap();
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
    assert_eq!(test_amount_irs(&params, string), 14)
}

#[test]
fn test_irs_no_gap_with_mismatches() {
    let params = SearchParams::new(10, 100, 0, 5).unwrap();
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
    assert_eq!(test_amount_irs(&params, string), 17)
}

#[test]
fn test_irs_no_mismatches_min_len_two() {
    let params = SearchParams::new(2, 100, 5, 0).unwrap();
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
    assert_eq!(test_amount_irs(&params, string), 58)
}

#[test]
fn test_irs_no_mismatches_min_len_two_no_gap() {
    let params = SearchParams::new(2, 100, 0, 0).unwrap();
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
    assert_eq!(test_amount_irs(&params, string), 18)
}

#[test]
fn test_irs_full_n_default_params() {
    let params = SearchParams::default();
    let string = "N".repeat(500);
    assert_eq!(test_amount_irs(&params, &string), 961)
}

#[test]
fn test_irs_full_n_custom_params() {
    let params = SearchParams::new(10, 100, 5, 1).unwrap();
    let string = "N".repeat(500);
    assert_eq!(test_amount_irs(&params, &string), 961)
}

#[test]
fn test_irs_full_n_no_gap() {
    let params = SearchParams::new(10, 100, 0, 1).unwrap();
    let string = "N".repeat(500);
    assert_eq!(test_amount_irs(&params, &string), 481)
}
