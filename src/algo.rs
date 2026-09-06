#![allow(clippy::many_single_char_names)]

use rmq::Rmq;

use crate::lcp_min::LcpMin;
use crate::{config::SearchParams, matrix::MatchMatrix};

/// Longest run of equal characters walked directly before falling back to the rmq.
const LCE_SCAN_LIMIT: usize = 8;

/// Build the LCP (Longest Common Prefix) array from a suffix array.
///
/// Employs a slightly modified version of the classic Kasai's algorithm.
pub fn lcp_array(s: &[u8], s_n: usize, sa: &[i32], inv_sa: &[usize]) -> Vec<u32> {
    // The values are bounded by the length of `s`, which divsufsort's i32 index type
    // already keeps below `i32::MAX`, so half of what `u32` holds.
    let mut lcp: Vec<u32> = vec![0; s_n];
    let mut j: usize;

    for i in 0..s_n {
        if inv_sa[i] != 0 {
            // Kasai's invariant: the lcp of a suffix is at most one shorter than the one
            // of the suffix starting a character before it. The first one starts fresh.
            let l = if i == 0 {
                0
            } else {
                lcp[inv_sa[i - 1]] as usize
            };
            j = l.saturating_sub(1);

            while s[i + j] == s[sa[inv_sa[i] - 1] as usize + j] {
                j += 1;
            }

            lcp[inv_sa[i]] = j as u32;
        }
    }

    lcp
}

/// Compute the mismatch location list.
///
/// Return the offsets of each mismatch found when extending outward from positions
/// `(i, j)` simultaneously, up to `mismatches` real mismatches (Kangaroo method).
///
/// A sentinel `0` is prepended. Mismatches within the first `initial_gap` characters
/// are recorded but do not consume the budget.
///
/// Note that because IUPAC matching is not transitive (A matches N, N matches G but A
/// doesn't match G), and our helper structures (lcp, rmq) were built on exact equality,
/// here we need to check at the frontier via match_u8.
//
// Notes:
// - Only considers "real" mismatches (degenerate string mismatching according to IUPAC
//   character matrix)
// - Longest Common Extension calculated from positions i and j
// - Simple explanation of the Kangaroo method: https://www.youtube.com/watch?v=Njv_q9RA-hs
// - For the BANANA case, the given (i, j) will be:
//     (1, 13), (1, 12), (2, 12), (2, 11), (3, 11) ... (6, 8)
#[allow(clippy::too_many_arguments)]
fn real_lce_mismatches<R: Rmq>(
    s: &[u8],
    i: usize,
    j: usize,
    inv_sa: &[usize],
    rmq: &LcpMin<R>,
    mut mismatches: i32,
    initial_gap: usize,
    matrix: &MatchMatrix,
) -> Vec<u32> {
    let s_n = s.len();
    let mut mismatch_locs = vec![0];
    let mut real_lce = 0;

    while mismatches >= 0 && j + real_lce != s_n {
        debug_assert!(i < j);
        // The rmq is constant time but its two rank lookups miss cache, and a shared
        // prefix is nearly always a character or two. So walk the frontier directly and
        // only pay for the rmq once a run gets long.
        let a = i + real_lce;
        let b = j + real_lce;
        if s[a] == s[b] {
            let mut k = 1;
            while k < LCE_SCAN_LIMIT && s[a + k] == s[b + k] {
                k += 1;
            }

            if k < LCE_SCAN_LIMIT {
                real_lce += k;
            } else {
                let (ii, jj) = (inv_sa[a], inv_sa[b]);
                let (lo, hi) = if ii < jj { (ii, jj) } else { (jj, ii) };
                real_lce += rmq.min(lo + 1, hi + 1);
            }
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

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Find all IRs in `seq` and return them as `(left, right, gap)` triples.
///
/// Recall that `s` is `seq` concatenated with its reverse complementary.
//
// Notes:
// - The original algorithm returned a set of tuples: BTreeSet<(i32, i32, i32)> but did no sorting.
//   It was marginally slower (compared to Vec<(i32, i32, i32)>, while making the code less clear.
//   >> AT NO POINT IS A DUPLICATE pushed into "irs".
// - If we use instead a Vec<(i32, i32, 32)> the collection needs to be returned sorted if the data
//   will be printed sorted afterwards in "format".
pub fn add_irs<R: Rmq + std::marker::Sync>(
    s: &[u8],
    inv_sa: &[usize],
    rmq: &LcpMin<R>,
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

/// Find all IRs centred at `c`.
///
/// Derives arm positions, collects mismatch locations via the Kangaroo method, then uses
/// a two-pointer sweep to emit every valid window within the mismatch budget and
/// `[min_len, max_len]` arm length, truncating overlong arms as needed.
fn add_irs_at_this_center<R: Rmq>(
    s: &[u8],
    n: usize,
    inv_sa: &[usize],
    rmq: &LcpMin<R>,
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
            // IR is too long, so truncate it, keeping the gap. The offset the truncation
            // lands on may be a mismatch, and an IR is not allowed to end on one, so
            // walk further in until it does not. There are at most `mismatches` of them
            // in a row.
            let mut truncated = start_mismatch + params.max_len;
            while truncated > start_mismatch
                && !matrix.match_u8(s[i + truncated - 1], s[j + truncated - 1])
            {
                truncated -= 1;
            }

            // Trimming those mismatches may have made the IR too short to report
            if truncated - start_mismatch < params.min_len {
                start_it_ptr += 1;
                continue;
            }

            let overshoot = end_mismatch - truncated;
            (left + overshoot, right - overshoot, gap)
        };

        irs_at_this_center.push(ir);

        start_it_ptr += 1;
    }

    irs_at_this_center
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The lcp of every suffix with the one preceding it in the suffix array, the slow way.
    fn brute_force_lcp(s: &[u8], sa: &[i32]) -> Vec<u32> {
        let mut lcp = vec![0; sa.len()];

        for rank in 1..sa.len() {
            let (a, b) = (sa[rank - 1] as usize, sa[rank] as usize);
            let mut k = 0;
            while a + k < s.len() && b + k < s.len() && s[a + k] == s[b + k] {
                k += 1;
            }
            lcp[rank] = k as u32;
        }

        lcp
    }

    #[test]
    fn test_lcp_array() {
        for s in [&b"acgtacgt#"[..], &b"nnnnn#"[..], &b"acgt$tgca#"[..]] {
            let sa: Vec<i32> = divsufsort::sort(s).into_parts().1;
            let mut inv_sa = vec![0; s.len()];
            for (i, value) in sa.iter().enumerate() {
                inv_sa[*value as usize] = i;
            }

            assert_eq!(lcp_array(s, s.len(), &sa, &inv_sa), brute_force_lcp(s, &sa));
        }
    }
}
