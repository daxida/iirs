use crate::{
    matrix::MatchMatrix,
    rmq::{Rmq, Sparse},
};

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

// Calculates a list of Longest Common Extensions, corresponding to 0, 1, 2, etc. allowed mismatches,
// up to maximum number of allowed mismatches.
//
// EXTRA INFO:
// - Only considers "real" mismatches (degenerate string mismatching according to IUPAC character matrix)
// - Takes into account the matching possibility of non A, C, G, T/U characters
// - Longest Common Extension calculated from positions i and j
// - Only starts counting number of allowed mismatches that occur after the given initial gap,
//   however earlier mismatches are still storeds
//
// - Kangaroo algorithm. A simple explanation can be found here: https://www.youtube.com/watch?v=Njv_q9RA-hs
// - For the BANANA case, the given (i, j) will be:
//     (1, 13), (1, 12), (2, 12), (2, 11), (3, 11) ... (6, 8)
//
#[allow(clippy::too_many_arguments)]
fn real_lce_mismatches(
    s: &[u8],
    i: usize,
    j: usize,
    s_n: usize,
    inv_sa: &[usize],
    rmq: &Sparse,
    mut mismatches: i32,
    initial_gap: usize,
    matrix: &MatchMatrix,
) -> Vec<u32> {
    let mut mismatch_locs = vec![0]; // Originally LinkedList<i32>
    let mut real_lce = 0;

    while mismatches >= 0 && j + real_lce != s_n {
        // LCE function in the original
        let ii = inv_sa[i + real_lce];
        let jj = inv_sa[j + real_lce];

        if ii < jj {
            // These are also valid for some reason (investigate why):
            // real_lce += rmq.rmq(ii - 1, jj + 1);
            // real_lce += rmq.rmq(ii, jj + 1);
            // real_lce += rmq.rmq(ii, jj + 2);
            // real_lce += rmq.rmq(ii, jj + 3);
            // real_lce += rmq.rmq(ii - 1, jj + 3);
            real_lce += rmq.rmq(ii, jj);
        }

        let ni = i + real_lce;
        let nj = j + real_lce;

        // if ni >= (s_n / 2) || nj >= s_n {
        if ni >= s_n / 2 {
            break;
        }

        if !matrix.match_u8(s[ni], s[nj]) {
            mismatch_locs.push((real_lce + 1) as u32);
            if real_lce + 1 >= initial_gap {
                mismatches -= 1;
            }
        }

        real_lce += 1;
    }

    mismatch_locs
}

// Finds all inverted repeats (palindromes) with given parameters and adds them to an output set
//
// NOTES:
// - The original algorithm returned a set of tuples: BTreeSet<(i32, i32, i32)> but did no sorting.
//   It was marginally slower (compared to Vec<(i32, i32, i32)>, while making the code less clear.
//   >> AT NO POINT IS A DUPLICATE pushed into "palindromes".
// - If we use instead a Vec<(i32, i32, 32)> the collection needs to be returned sorted if the data will be printed sorted
//   afterwards in "format". The palindromes found are the same without sorting, they are just not returned in the expected order.
use rayon::prelude::*;
#[allow(clippy::too_many_arguments)]
pub fn add_palindromes(
    s: &[u8],
    s_n: usize,
    n: usize,
    inv_sa: &[usize],
    rmq: &Sparse,
    min_len: usize,
    max_len: usize,
    mismatches: usize,
    max_gap: usize,
    matrix: &MatchMatrix,
) -> Vec<(usize, usize, usize)> {
    (0..=2 * (n - 1))
        .into_par_iter()
        .flat_map(|c| {
            add_palindromes_at_this_center(
                s, s_n, n, inv_sa, rmq, min_len, max_len, mismatches, max_gap, matrix, c,
            )
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn add_palindromes_at_this_center(
    s: &[u8],
    s_n: usize,
    n: usize,
    inv_sa: &[usize],
    rmq: &Sparse,
    min_len: usize,
    max_len: usize,
    mismatches: usize,
    max_gap: usize,
    matrix: &MatchMatrix,
    c: usize,
) -> Vec<(usize, usize, usize)> {
    let mut palindromes_at_this_center = Vec::new();

    let c = (c as f64) / 2.0;

    // This could be computed outside of the loop
    let behind = (2 * n + 1) as f64;
    let is_max_gap_odd = max_gap % 2 == 1;
    let half_gap = max_gap / 2;

    // Determine if value of centre corresponds to an odd or even palindrome
    let is_odd = c.fract() == 0.0;

    // The following part is equivalent to:
    // let margin = c.fract();
    // let (i, j) = (1.0 + c - margin, behind - c - margin);
    let (i, j) = if is_odd {
        ((c + 1.0) as usize, (behind - c) as usize)
    } else {
        ((c + 0.5) as usize, (behind - (c + 0.5)) as usize)
    };

    // We add 1 compared to the original implementation to guarantee >= 0
    let initial_gap = if is_max_gap_odd || !is_odd {
        half_gap + 1
    } else {
        half_gap
    };

    let mismatch_locs = real_lce_mismatches(
        s,
        i,
        j,
        s_n,
        inv_sa,
        rmq,
        mismatches as i32,
        initial_gap,
        matrix,
    );

    // Get a list of valid start and end mismatch locations
    // (that could mark the potential start or end of a palindrome)
    let mut valid_start_locs = Vec::new();
    let mut valid_end_locs = Vec::new();
    let sz = mismatch_locs.len();

    for (id, loc) in mismatch_locs.iter().enumerate() {
        if id < sz - 1 && mismatch_locs[id + 1] != *loc + 1 {
            valid_start_locs.push((*loc, id));
            valid_end_locs.push((mismatch_locs[id + 1], id + 1));
        }
    }

    // If there are no valid starts, there should not be valid ends.
    debug_assert!(valid_start_locs.is_empty() || !valid_end_locs.is_empty());

    let mut start_it_ptr = 0;
    let mut end_it_ptr = 0;

    while start_it_ptr < valid_start_locs.len() && end_it_ptr < valid_end_locs.len() {
        let mut start = valid_start_locs[start_it_ptr];
        let mut end = valid_end_locs[end_it_ptr];

        // Count the difference in mismatches between the start and end location
        let mut mismatch_diff = end.1 - start.1 - 1;

        // While mismatch difference is too large, move start location to the right
        while mismatch_diff > mismatches {
            start_it_ptr += 1;
            start = valid_start_locs[start_it_ptr];
            mismatch_diff = end.1 - start.1 - 1;
        }

        let start_mismatch = start.0 as usize;
        if start_mismatch >= initial_gap {
            break;
        }

        // While mismatch difference is within acceptable bound, move end location to the right
        while mismatch_diff <= mismatches {
            end_it_ptr += 1;
            if end_it_ptr == valid_end_locs.len() {
                break;
            }
            end = valid_end_locs[end_it_ptr];
            mismatch_diff = end.1 - start.1 - 1;
        }

        debug_assert!(end_it_ptr > start_it_ptr);
        // And since start_it_ptr >= 0 because usize, we have: end_it_ptr > 0

        let end_mismatch = valid_end_locs[end_it_ptr - 1].0 - 1;

        let palindrome_length = end_mismatch as usize - start_mismatch;
        if palindrome_length < min_len {
            start_it_ptr += 1;
            continue;
        }

        // The following part is equivalent to:
        // let margin = c.fract();
        // let left  = (c + margin) as usize - end_mismatch;
        // let right = (c - margin) as usize + end_mismatch;
        // let gap   = 2 * start_mismatch + 1 - (margin * 2.0) as usize;
        //
        // NOTE: we find again that:
        // let palindrome_length = (right - left + 1 - gap) / 2;
        let (left, right, gap): (usize, usize, usize);

        if is_odd {
            left = (c - end_mismatch as f64) as usize;
            right = (c + end_mismatch as f64) as usize;
            gap = 2 * start_mismatch + 1;
        } else {
            left = (c - 0.5 - (end_mismatch as f64 - 1.0)) as usize;
            right = (c + 0.5 + (end_mismatch as f64 - 1.0)) as usize;
            gap = 2 * start_mismatch;
        }

        let palindrome = if palindrome_length <= max_len {
            // Palindrome is not too long, so add to output
            (left, right, gap)
        } else {
            // Palindrome is too long, so attempt truncation
            let overshoot = palindrome_length - max_len;

            // Check if truncation results in the potential palindrome ending in a mismatch
            if overshoot != 0 {
                // Potential palindrome does not end in a mismatch, so add to output
                (left + overshoot, right - overshoot, gap)
            } else {
                // Potential palindrome does end in a mismatch, so truncate a character
                (left + overshoot + 1, right - overshoot - 1, gap)
            }
        };

        palindromes_at_this_center.push(palindrome);

        start_it_ptr += 1;
    }

    palindromes_at_this_center
}
