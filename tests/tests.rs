use iirs::{RepeatType, SearchParams, find_repeats};

fn test_amount_repeats(params: &SearchParams, string: &str) -> usize {
    let seq = string.to_ascii_lowercase().as_bytes().to_vec();
    params.check_bounds(seq.len()).unwrap();
    find_repeats(params, &seq).unwrap().len()
}

#[test]
fn test_mirror_repeats() {
    // "aacc" followed by its reverse, "ccaa".
    let seq = "aaccctccaa".as_bytes();
    let params = SearchParams::new(4, 4, 2, 0).unwrap();
    let mirror = params.clone().with_repeat_type(RepeatType::Mirror);
    assert_eq!(find_repeats(&params, seq).unwrap(), vec![]);
    assert_eq!(find_repeats(&mirror, seq).unwrap(), vec![(0, 9, 2)]);
}

#[test]
fn test_direct_repeats() {
    let params = SearchParams::new(4, 4, 2, 0).unwrap();
    let direct = params.clone().with_repeat_type(RepeatType::Direct);
    let complement = params.with_repeat_type(RepeatType::Complement);

    assert_eq!(
        find_repeats(&direct, "aaccctaacc".as_bytes()).unwrap(),
        vec![(0, 9, 2)]
    );
    assert_eq!(
        find_repeats(&complement, "aaccctttgg".as_bytes()).unwrap(),
        vec![(0, 9, 2)]
    );
    assert_eq!(
        find_repeats(&direct, "aaccctttgg".as_bytes()).unwrap(),
        vec![]
    );
}

#[test]
fn test_direct_repeats_do_not_overlap() {
    // "acac" ... "acac" fits, and so does "ac" ... "ac" two apart, truncated to the shift.
    // Nothing longer: the two arms of a repeat may not overlap.
    let params = SearchParams::new(2, 100, 0, 0)
        .unwrap()
        .with_repeat_type(RepeatType::Direct);

    let seq = "acacacac".as_bytes();
    assert_eq!(
        find_repeats(&params, seq).unwrap(),
        vec![(0, 7, 0), (0, 3, 0)]
    );
}

#[test]
fn test_irs_default_params() {
    let params = SearchParams::default();
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA".repeat(100);
    assert_eq!(test_amount_repeats(&params, &string), 10068);
}

#[test]
fn test_irs_custom_params() {
    let params = SearchParams::new(10, 100, 5, 1).unwrap();
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
    assert_eq!(test_amount_repeats(&params, string), 21);
}

#[test]
fn test_irs_no_mismatches() {
    let params = SearchParams::new(10, 100, 5, 0).unwrap();
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
    assert_eq!(test_amount_repeats(&params, string), 14);
}

#[test]
fn test_irs_no_gap_with_mismatches() {
    let params = SearchParams::new(10, 100, 0, 5).unwrap();
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
    assert_eq!(test_amount_repeats(&params, string), 17);
}

#[test]
fn test_irs_max_gap_with_mismatches() {
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
    let params = SearchParams::new(10, 100, string.len() - 1, 5).unwrap();
    assert_eq!(test_amount_repeats(&params, string), 54);

    let params = SearchParams::new(10, 100, string.len(), 5).unwrap();
    assert_eq!(test_amount_repeats(&params, string), 54);
}

#[test]
fn test_irs_huge_gap_with_mismatches() {
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
    let params = SearchParams::new(10, 100, 100, 5).unwrap();
    assert_eq!(test_amount_repeats(&params, string), 54);

    let params = SearchParams::new(10, 100, 101, 5).unwrap();
    assert_eq!(test_amount_repeats(&params, string), 54);
}

#[test]
fn test_irs_max_max_gap_with_mismatches() {
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
    let params = SearchParams::new(10, 100, usize::MAX, 5).unwrap();
    assert_eq!(test_amount_repeats(&params, string), 54);
}

#[test]
fn test_irs_no_mismatches_min_len_two() {
    let params = SearchParams::new(2, 100, 5, 0).unwrap();
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
    assert_eq!(test_amount_repeats(&params, string), 58);
}

#[test]
fn test_irs_no_mismatches_min_len_two_no_gap() {
    let params = SearchParams::new(2, 100, 0, 0).unwrap();
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";
    assert_eq!(test_amount_repeats(&params, string), 18);
}

#[test]
fn test_irs_full_n_default_params() {
    let params = SearchParams::default();
    let string = "N".repeat(500);
    assert_eq!(test_amount_repeats(&params, &string), 961);
}

#[test]
fn test_irs_full_n_custom_params() {
    let params = SearchParams::new(10, 100, 5, 1).unwrap();
    let string = "N".repeat(500);
    assert_eq!(test_amount_repeats(&params, &string), 961);
}

#[test]
fn test_irs_full_n_no_gap() {
    let params = SearchParams::new(10, 100, 0, 1).unwrap();
    let string = "N".repeat(500);
    assert_eq!(test_amount_repeats(&params, &string), 481);
}

//
// The other repeat types
//

#[test]
fn test_amount_of_repeats_of_each_type() {
    let string = "AGUCSGTWGTGTGTWKMMMKKBDDN-NN*HAGTTWGuVVVNNAGuGTA";

    // how many of each of the four kinds, in order, the sequence holds
    let cases = [
        (
            SearchParams::default(),
            string.repeat(100),
            [10068, 13138, 13531, 7698],
        ),
        (
            SearchParams::new(10, 100, 5, 1).unwrap(),
            string.to_string(),
            [21, 25, 20, 13],
        ),
    ];

    for (params, seq, expected) in cases {
        let types = [
            RepeatType::Inverted,
            RepeatType::Mirror,
            RepeatType::Direct,
            RepeatType::Complement,
        ];
        for (repeat_type, expected) in types.into_iter().zip(expected) {
            let params = params.clone().with_repeat_type(repeat_type);
            assert_eq!(
                test_amount_repeats(&params, &seq),
                expected,
                "{repeat_type}"
            );
        }
    }
}
