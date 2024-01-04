use std::collections::BTreeSet;

use crate::{rmq::rmq, matrix::MatchMatrix};

// Calculates the Longest Common Prefix array of a text and stores value in given variable LCP
//
// INPUT:
// - Text
// - Text length
// - Suffix Array
// - Longest Common Prefix data structure (empty)
pub fn lcp_array(s: &[u8], s_n: usize, sa: &[i64], inv_sa: &[usize]) -> Vec<usize> {
    let mut lcp: Vec<usize> = vec![0; s_n];
    let mut j: usize;

    for i in 1..s_n {
        if inv_sa[i] != 0 {
            let l = lcp[inv_sa[i - 1]];
            j = if l > 1 { l - 1 } else { 0 };

            while s[i + j] == s[sa[inv_sa[i] - 1] as usize + j] {
                j += 1;
            }

            lcp[inv_sa[i]] = j;
        }
    }

    lcp
}

// Returns the Longest Common Extension between position i and j (order of i, j input does not matter)
//
// INPUT:
// - Indexes i and j
// - Text length
// - Inverse Suffix Array
// - Longest Common Prefix Array data structure (filled)
// - Data structure (filled) with preprocessed values to perform Range Minimum Queries (Type 1: 'A', Type 2: 'rmq')
fn lce(
    i: usize,
    j: usize,
    s_n: usize,
    inv_sa: &[usize],
    lcp: &[usize],
    rmq_prep: &[usize],
) -> usize {
    if j == s_n {
        return 0;
    }

    assert!(i < j);

    lcp[rmq(rmq_prep, lcp, s_n, inv_sa[i], inv_sa[j])]
}

// Calculates a list of Longest Common Extensions, corresponding to 0, 1, 2, etc. allowed mismatches, up to maximum number of allowed mismatches
//
// EXTRA INFO:
// - Only considers "real" mismatches (degenerate string mismatching according to IUPAC character matrix)
// - Takes into account the matching possibility of non A, C, G, T/U characters
// - Longest Common Extension calculated from positions i and j (order of i, j input does not matter)
// - Only starts counting number of allowed mismatches that occur after the given initial gap, however earlier mismatches are still stored
// - Should only be used after MatchMatrix has been instantiated with necessary data
//
// INPUT:
// - Text
// - Indexes i and j
// - Text length
// - Inverse Suffix Array
// - Longest Common Prefix Array (LCP)
// - Data structure (filled) with preprocessed values to perform Range Minimum Queries (Type 1: 'A', Type 2: 'rmq')
// - Maximum number of allowed mismatches
// - Initial gap
// - Data structure to store resulting mismatch locations
#[allow(clippy::too_many_arguments)]
fn real_lce_mismatches(
    s: &[u8],
    i: usize,
    j: usize,
    s_n: usize,
    inv_sa: &[usize],
    lcp: &[usize],
    rmq_prep: &[usize],
    mut mismatches: i32,
    initial_gap: i32,
    matrix: &MatchMatrix,
) -> Vec<i32> {
    let mut mismatch_locs = Vec::new(); // Originally LinkedList<i32>
    mismatch_locs.push(-1);

    let mut real_lce = 0;
    while mismatches >= 0 {
        real_lce += lce(i + real_lce, j + real_lce, s_n, inv_sa, lcp, rmq_prep);

        let ni = i + real_lce;
        let nj = j + real_lce;

        if ni >= (s_n / 2) || nj >= s_n {
            break;
        }

        if !matrix.match_u8(s[ni], s[nj]) {
            mismatch_locs.push(real_lce as i32);
            if real_lce >= initial_gap as usize {
                mismatches -= 1;
            }
        }

        real_lce += 1;
    }

    mismatch_locs
}

// Finds all inverted repeats (palindromes) with given parameters and adds them to an output set
//
// INPUT:
// - Data structure (set of integer 3-tuples) to store palindromes in form (left_index, right_index, gap)
// - S = text + '$' + complement(reverse(text) + '#'
// - Length of S
// - Text length
// - Inverse Suffix Array
// - Longest Common Prefix Array (LCP)
// - Data structure (filled) with preprocessed values to perform Range Minimum Queries (Type 1: 'A', Type 2: 'rmq')
// - Tuple of parameters for palindromes to be found (minimum_length, maximum_length, maximum_allowed_number_of_mismatches, maximum_gap)
#[allow(clippy::too_many_arguments)]
pub fn add_palindromes(
    s: &[u8],
    s_n: usize,
    n: usize,
    inv_sa: &[usize],
    lcp: &[usize],
    rmq_prep: &[usize],
    min_len: i32,
    max_len: i32,
    mismatches: i32,
    max_gap: i32,
    matrix: &MatchMatrix,
) -> BTreeSet<(i32, i32, i32)> {
    let mut palindromes: BTreeSet<(i32, i32, i32)> = BTreeSet::new();
    let behind = (2 * n + 1) as f64;

    for c in (0..=2 * (n - 1)).map(|c| (c as f64) / 2.0) {
        let is_odd = c.fract() == 0.0;

        let (i, j) = if is_odd {
            (c + 1.0, behind - c)
        } else {
            (c + 0.5, behind - (c + 0.5))
        };

        let initial_gap = if max_gap % 2 == 1 {
            (max_gap - 1) / 2
        } else if is_odd {
            (max_gap - 2) / 2
        } else {
            max_gap / 2
        };

        let mismatch_locs = real_lce_mismatches(
            s,
            i as usize,
            j as usize,
            s_n,
            inv_sa,
            lcp,
            rmq_prep,
            mismatches,
            initial_gap,
            matrix,
        );

        let mut valid_start_locs: Vec<(i32, i32)> = Vec::new();
        let mut valid_end_locs: Vec<(i32, i32)> = Vec::new();

        // Determine list of valid start and end mismatch locations
        // (that could mark the potential start or end of a palindrome)
        let mut mismatch_id = 0;
        let mut prev: Option<&i32> = None;
        let mut iter = mismatch_locs.iter().peekable();
        while let Some(current) = iter.next() {
            if let Some(&next) = iter.peek() {
                if *next != *current + 1 {
                    valid_start_locs.push((*current, mismatch_id));
                }
            }
            if let Some(prev) = prev {
                if *prev != *current - 1 {
                    valid_end_locs.push((*current, mismatch_id));
                }
            }
            prev = Some(current);
            mismatch_id += 1;
        }

        if valid_start_locs.is_empty() || valid_end_locs.is_empty() {
            continue;
        }

        let mut start_it_ptr = 0;
        let mut end_it_ptr = 0;
        let mut mismatch_diff: i32;
        let mut start_mismatch: i32;
        let mut end_mismatch: i32;

        while start_it_ptr < valid_start_locs.len() && end_it_ptr < valid_end_locs.len() {
            let mut start = valid_start_locs[start_it_ptr];
            let mut end = valid_end_locs[end_it_ptr];

            // Count the difference in mismatches between the start and end location
            mismatch_diff = end.1 - start.1 - 1;

            // While mismatch difference is too large,
            // move start location to the right until mismatch difference is within acceptable bound
            while mismatch_diff > mismatches {
                start_it_ptr += 1;
                start = valid_start_locs[start_it_ptr];
                mismatch_diff = end.1 - start.1 - 1;
            }

            // While mismatch difference is within acceptable bound,
            // move end location to the right until mismatch difference becomes unacceptable
            while mismatch_diff <= mismatches {
                end_it_ptr += 1;
                if end_it_ptr == valid_end_locs.len() {
                    break;
                }
                end = valid_end_locs[end_it_ptr];
                mismatch_diff = end.1 - start.1 - 1;
            }

            start_mismatch = start.0;
            end_mismatch = valid_end_locs[end_it_ptr - 1].0;

            // println!("(start_mismatch, end_mismatch) = {} {}", start_mismatch, end_mismatch);

            // // Skip this iteration if the start mismatch chosen is such that the gap is not within the acceptable bound
            if start_mismatch >= initial_gap {
                break;
            }

            let left: i32;
            let right: i32;
            let gap: i32;

            if is_odd {
                left = (c - end_mismatch as f64) as i32;
                right = (c + end_mismatch as f64) as i32;
                gap = 2 * (start_mismatch + 1) + 1;
            } else {
                left = (c - 0.5 - (end_mismatch as f64 - 1.0)) as i32;
                right = (c + 0.5 + (end_mismatch as f64 - 1.0)) as i32;
                gap = 2 * (start_mismatch + 1);
            }

            // println!("(left, gap, right) = {} {} {}", left, right, gap);

            // Check that potential palindrome is not too short
            if (right - left + 1 - gap) / 2 >= min_len {
                // Check that potential palindrome is not too long
                let palindrome = if (right - left + 1 - gap) / 2 <= max_len {
                    // Palindrome is not too long, so add to output
                    (left, right, gap)
                } else {
                    // Palindrome is too long, so attempt truncation
                    let prev_end_mismatch_ptr = (end_it_ptr as i32 - 2).max(0) as usize;
                    let prev_end_mismatch = valid_end_locs[prev_end_mismatch_ptr].0;
                    let mismatch_gap = end_mismatch - prev_end_mismatch - 1;
                    let overshoot = ((right - left + 1 - gap) / 2) - max_len;

                    // Check if truncation results in the potential palindrome ending in a mismatch
                    if overshoot != mismatch_gap {
                        // Potential palindrome does not end in a mismatch, so add to output
                        (left + overshoot, right - overshoot, gap)
                    } else {
                        // Potential palindrome does end in a mismatch, so truncate an additional 1
                        // character either side then add to output
                        (left + overshoot + 1, right - overshoot - 1, gap)
                    }
                };
                palindromes.insert(palindrome);
            }

            start_it_ptr += 1;
        }
    }

    palindromes
}
