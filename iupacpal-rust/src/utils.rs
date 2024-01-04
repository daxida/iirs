use std::cmp::Ordering;
// use linked_list::LinkedList;
use std::collections::BTreeSet;

use crate::matrix::MatchMatrix;

#[inline(always)]
fn flog2(v: usize) -> usize {
    v.ilog2() as usize
}

///////////////////////////////////////////
//  RANGE MINIMUM QUERY (RMQ) FUNCTIONS  //
///////////////////////////////////////////

// Range Minimum Query (Type 1)
// GETS CALLED LIKE THIS: LCP[rmq(A, LCP, n, a, b)] -- only in LCE
fn rmq(ary: &[usize], lcp: &[usize], n: usize, mut i: usize, mut j: usize) -> usize {
    let lgn = flog2(n);

    if i > j {
        std::mem::swap(&mut i, &mut j);
    }

    i += 1;

    match i.cmp(&j) {
        Ordering::Greater => 0,
        Ordering::Equal => i,
        Ordering::Less => {
            assert!(i < j);
            assert!(j - i + 1 > 0);
            let k = flog2(j - i + 1);
            let a = ary[i * lgn + k];
            let b = ary[(j - (1 << k) + 1) * lgn + k];

            if lcp[a] > lcp[b] {
                b
            } else {
                a
            }
        }
    }
}

// O(nlogn)-time preprocessing function for Type 1 Range Minimum Queries
// It is going to be called with these args: rmq_preprocess(&lcp, s_n)
pub fn rmq_preprocess(lcp: &[usize], n: usize) -> Vec<usize> {
    let lgn = flog2(n);
    let mut ary = vec![0; n * lgn];

    for i in 0..n {
        ary[i * lgn] = i;
    }

    let mut j = 1;
    while (1 << j) <= n {
        for i in 0..=n - (1 << j) {
            let idx_1 = i * lgn + j;
            let idx_2 = (i + (1 << (j - 1))) * lgn + j - 1;
            ary[idx_1] = if lcp[ary[idx_1 - 1]] < lcp[ary[idx_2]] {
                ary[idx_1 - 1]
            } else {
                ary[idx_2]
            }
        }
        j += 1;
    }

    ary
}

////////////////////////
//  STRING FUNCTIONS  //
////////////////////////

// Calculates the Longest Common Prefix array of a text and stores value in given variable LCP
//
// INPUT:
// - Text
// - Text length
// - Suffix Array
// - Longest Common Prefix data structure (empty)
pub fn lcp_array(s: &[u8], n: usize, sa: &[i64], inv_sa: &[usize]) -> Vec<usize> {
    let mut lcp: Vec<usize> = vec![0; n];
    let mut j: usize;

    for i in 0..n {
        if inv_sa[i] != 0 {
            if i == 0 {
                j = 0;
            } else {
                let tmp = lcp[inv_sa[i - 1]];
                j = if tmp >= 2 { tmp - 1 } else { 0 };
            }

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
fn lce(i: usize, j: usize, n: usize, inv_sa: &[usize], lcp: &[usize], ary: &[usize]) -> usize {
    if i == j {
        return n - i;
    }
    if j == n {
        return 0;
    }

    assert!(i < j);

    lcp[rmq(ary, lcp, n, inv_sa[i], inv_sa[j])]
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
pub fn real_lce_mismatches(
    text: &[u8],
    i: usize,
    j: usize,
    n: usize,
    inv_sa: &[usize],
    lcp: &[usize],
    ary: &[usize],
    mut mismatches: i32,
    initial_gap: i32,
    matrix: &MatchMatrix,
) -> Vec<i32> {
    let mut mismatch_locs = Vec::new(); // Originally LinkedList<i32>
    mismatch_locs.insert(0, -1); 

    if i == j {
        mismatch_locs.push((n - i) as i32);
    } else {
        let mut real_lce = 0;

        while mismatches >= 0 {
            real_lce += lce(i + real_lce, j + real_lce, n, inv_sa, lcp, ary);

            let ni = i + real_lce;
            let nj = j + real_lce;

            if ni >= (n / 2) || nj >= n {
                break;
            }

            let s1 = text[ni];
            let s2 = text[nj];

            if !matrix.match_chars(s1 as char, s2 as char) {
                mismatch_locs.push(real_lce as i32);
                if real_lce >= initial_gap as usize {
                    mismatches -= 1;
                }
            }

            real_lce += 1;
        }
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
    ary: &[usize],
    min_len: i32,
    max_len: i32,
    mismatches: i32,
    max_gap: i32,
    matrix: &MatchMatrix,
) -> BTreeSet<(i32, i32, i32)> {
    let mut palindromes: BTreeSet<(i32, i32, i32)> = BTreeSet::new();

    for c in (0..=2 * (n - 1)).map(|c| (c as f64) / 2.0) {
        let is_odd = c.fract() == 0.0;

        let (i, j) = if is_odd {
            (c as i32 + 1, (2.0 * n as f64 + 1.0 - c) as i32)
        } else {
            ((c + 0.5) as i32, (2.0 * n as f64 + 1.0 - (c + 0.5)) as i32)
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
            ary,
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

        // Optional printing of mismatch locations relative to centre, valid start locations
        // and valid end locations
        // if false {
        //     println!("centre = {}", c);
        //     println!("mismatches: \t[ {} ]",
        //         mismatch_locs
        //             .iter()
        //             .map(|loc| format!("{} ", loc))
        //             .collect::<String>()
        //     );
        //     println!("starts: \t[ {} ]",
        //         valid_start_locs
        //             .iter()
        //             .map(|(cur, id)| format!("({}, {}) ", cur, id))
        //             .collect::<String>()
        //     );
        //     println!("ends: \t\t[ {} ]",
        //         valid_end_locs
        //             .iter()
        //             .map(|(cur, id)| format!("({}, {}) ", cur, id))
        //             .collect::<String>()
        //     );
        // }

        if !valid_start_locs.is_empty() && !valid_end_locs.is_empty() {
            // let mut start_it = valid_start_locs.iter().clone();
            // let mut end_it = valid_end_locs.iter().clone();
            let mut start_it_ptr = 0;
            let mut end_it_ptr = 0;

            let mut mismatch_diff: i32;
            let mut left: i32;
            let mut right: i32;
            let mut gap: i32;
            let mut start_mismatch: i32;
            let mut end_mismatch: i32;

            // Loop while both start and end mismatch locations have not reached the end of their respective lists
            while let (Some(mut start), Some(mut end)) = (
                valid_start_locs.get(start_it_ptr).copied(), // Get a copy of the element or None
                valid_end_locs.get(end_it_ptr).copied(),     // Get a copy of the element or None
            ) {
                // while let (Some(mut start), Some(mut end)) = (start_it.next(), end_it.next()) {
                // Count the difference in mismatches between the start and end location
                mismatch_diff = end.1 - start.1 - 1;

                // While mismatch difference is too large,
                // move start location to the right until mismatch difference is within acceptable bound
                while mismatch_diff > mismatches {
                    start_it_ptr += 1;
                    let next_start = valid_start_locs.get(start_it_ptr).copied().unwrap();
                    start = next_start;
                    mismatch_diff = end.1 - start.1 - 1;
                }

                // While mismatch difference is within acceptable bound,
                // move end location to the right until mismatch difference becomes unacceptable
                while mismatch_diff <= mismatches {
                    end_it_ptr += 1;
                    if end_it_ptr == valid_end_locs.len() {
                        // Tis never read because we also exit the nester While
                        // mismatch_diff = 0 - start.1 - 1;
                        break;
                    } else {
                        let next_end = valid_end_locs.get(end_it_ptr).copied().unwrap();
                        end = next_end;
                        mismatch_diff = end.1 - start.1 - 1;
                    }
                }

                start_mismatch = start.0; // Pick the current start mismatch
                let prev_end_it_ptr = end_it_ptr - 1;
                end_mismatch = valid_end_locs.get(prev_end_it_ptr).copied().unwrap().0;

                // println!("(start_mismatch, end_mismatch) = {} {}", start_mismatch, end_mismatch);

                // // Skip this iteration if the start mismatch chosen is such that the gap is not within the acceptable bound
                if start_mismatch >= initial_gap {
                    break;
                }

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
                    if (right - left + 1 - gap) / 2 <= max_len {
                        // Palindrome is not too long, so add to output
                        palindromes.insert((left, right, gap));
                    } else {
                        // Palindrome is too long, so attempt truncation
                        let prev_end_mismatch_ptr = (end_it_ptr as i32 - 2).max(0) as usize;
                        let prev_end_mismatch = valid_end_locs
                            .get(prev_end_mismatch_ptr)
                            .copied()
                            .unwrap()
                            .0;
                        let mismatch_gap = end_mismatch - prev_end_mismatch - 1;
                        let overshoot = ((right - left + 1 - gap) / 2) - max_len;

                        // Check if truncation results in the potential palindrome ending in a mismatch
                        if overshoot != mismatch_gap {
                            // Potential palindrome does not end in a mismatch, so add to output
                            palindromes.insert((left + overshoot, right - overshoot, gap));
                        } else {
                            // Potential palindrome does end in a mismatch, so truncate an additional 1
                            // character either side then add to output
                            palindromes.insert((left + overshoot + 1, right - overshoot - 1, gap));
                        }
                    }
                }

                start_it_ptr += 1;
            }
        }
    }

    palindromes
}
