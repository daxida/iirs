use rmq::Rmq;

use crate::{config::SearchParams, matrix::MatchMatrix};

pub fn lcp_array(s: &[u8], s_n: usize, sa: &[i32], inv_sa: &[usize]) -> Vec<usize> {
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
fn real_lce_mismatches<R: Rmq>(
    s: &[u8],
    i: usize,
    j: usize,
    inv_sa: &[usize],
    rmq: &R,
    mut mismatches: i32,
    initial_gap: usize,
    matrix: &MatchMatrix,
) -> Vec<u32> {
    let s_n = s.len();
    let mut mismatch_locs = vec![0];
    let mut real_lce = 0;

    while mismatches >= 0 && j + real_lce != s_n {
        // LCE function in the original
        let ii = inv_sa[i + real_lce];
        let jj = inv_sa[j + real_lce];

        if ii < jj {
            real_lce += rmq.rmq(ii + 1, jj + 1).unwrap_or(0);
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

// TODO: Clear this
//
// Finds all inverted repeats (IRs) with given parameters and adds them to an output set
//
// NOTES:
// - The original algorithm returned a set of tuples: BTreeSet<(i32, i32, i32)> but did no sorting.
//   It was marginally slower (compared to Vec<(i32, i32, i32)>, while making the code less clear.
//   >> AT NO POINT IS A DUPLICATE pushed into "irs".
// - If we use instead a Vec<(i32, i32, 32)> the collection needs to be returned sorted if the data
//   will be printed sorted afterwards in "format".
#[cfg(feature = "parallel")]
use rayon::prelude::*;

pub fn add_irs<R: Rmq + std::marker::Sync>(
    s: &[u8],
    inv_sa: &[usize],
    rmq: &R,
    params: &SearchParams,
    matrix: &MatchMatrix,
) -> Vec<(usize, usize, usize)> {
    let s_n = s.len();
    let n = s_n / 2 - 1;

    // Conditional compilation for parallel execution
    #[cfg(feature = "parallel")]
    let result: Vec<_> = (params.min_len..(s_n - 1 - params.min_len))
        .into_par_iter()
        .flat_map(|c| add_irs_at_this_center(s, n, inv_sa, rmq, params, matrix, c))
        .collect();

    // Conditional compilation for sequential execution
    #[cfg(not(feature = "parallel"))]
    let result: Vec<_> = (params.min_len..(s_n - 1 - params.min_len))
        .flat_map(|c| add_irs_at_this_center(s, n, inv_sa, rmq, params, matrix, c))
        .collect();

    result
}

fn add_irs_at_this_center<R: Rmq>(
    s: &[u8],
    n: usize,
    inv_sa: &[usize],
    rmq: &R,
    params: &SearchParams,
    matrix: &MatchMatrix,
    c: usize,
) -> Vec<(usize, usize, usize)> {
    let mut irs_at_this_center = Vec::new();

    // This could be computed outside of the loop.
    // It is done inside to ease the parallel / sequential structure.
    let behind = (2 * n + 1) as f64;
    let is_max_gap_odd = params.max_gap % 2 == 1;
    let half_gap = params.max_gap / 2;

    // Note that the current IR is odd iif margin is equal to zero
    let c = (c as f64) / 2.0;
    let margin = c.fract();

    // We add 1 compared to the original implementation to guarantee >= 0
    let initial_gap = if is_max_gap_odd {
        half_gap + 1
    } else {
        half_gap + (2.0 * margin) as usize
    };

    let i = (1.0 + c - margin) as usize;
    let j = (behind - c - margin) as usize;

    let mismatch_locs = real_lce_mismatches(
        s,
        i,
        j,
        inv_sa,
        rmq,
        params.mismatches as i32,
        initial_gap,
        matrix,
    );

    // Get a list of valid start and end mismatch locations
    // (that could mark the potential start or end of an IR)
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
        while mismatch_diff > params.mismatches {
            start_it_ptr += 1;
            start = valid_start_locs[start_it_ptr];
            mismatch_diff = end.1 - start.1 - 1;
        }

        let start_mismatch = start.0 as usize;
        if start_mismatch >= initial_gap {
            break;
        }

        // While mismatch difference is within acceptable bound, move end location to the right
        while mismatch_diff <= params.mismatches {
            end_it_ptr += 1;
            if end_it_ptr == valid_end_locs.len() {
                break;
            }
            end = valid_end_locs[end_it_ptr];
            mismatch_diff = end.1 - start.1 - 1;
        }

        debug_assert!(end_it_ptr > start_it_ptr);
        // And since start_it_ptr >= 0 because usize, we have: end_it_ptr > 0

        let end_mismatch = (valid_end_locs[end_it_ptr - 1].0 - 1) as usize;

        let ir_length = end_mismatch - start_mismatch;
        if ir_length < params.min_len {
            start_it_ptr += 1;
            continue;
        }

        let left = (c + margin) as usize - end_mismatch;
        let right = (c - margin) as usize + end_mismatch;
        let gap = 2 * start_mismatch + 1 - (2.0 * margin) as usize;
        debug_assert!(gap <= params.max_gap);

        let ir = if ir_length <= params.max_len {
            // IR is not too long, so add to output
            (left, right, gap)
        } else {
            // IR is too long, so attempt truncation
            let overshoot = ir_length - params.max_len;

            let prev_ptr = (end_it_ptr as i32 - 2).max(0) as usize;
            let prev = (valid_end_locs[prev_ptr].0 - 1) as usize;
            let mismatch_gap = if end_mismatch == prev {
                0
            } else {
                end_mismatch - prev - 1
            };

            // Check if truncation results in the potential IR ending in a mismatch
            if overshoot != mismatch_gap {
                // Potential IR does not end in a mismatch, so add to output
                (left + overshoot, right - overshoot, gap)
            } else {
                // Potential IR does end in a mismatch, so truncate a character
                (left + overshoot + 1, right - overshoot - 1, gap)
            }
        };

        irs_at_this_center.push(ir);

        start_it_ptr += 1;
    }

    irs_at_this_center
}
