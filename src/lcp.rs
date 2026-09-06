//! Construction of the LCP (Longest Common Prefix) array.

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
