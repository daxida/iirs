#[inline(always)]
fn flog2(v: usize) -> usize {
    v.ilog2() as usize
}

// Range Minimum Query.
pub fn rmq(rmq_prep: &[usize], lcp: &[usize], s_n: usize, mut i: usize, j: usize) -> usize {
    debug_assert!(i < j);
    i += 1;
    
    if i < j {
        let lgn = flog2(s_n);
        let k = flog2(j - i + 1);

        // Calculate indices a and b for the two halves of the range.
        let idx_a = i * lgn + k;
        let idx_b = (j - (1 << k) + 1) * lgn + k;
        let a = rmq_prep[idx_a];
        let b = rmq_prep[idx_b];

        if lcp[a] > lcp[b] {
            b
        } else {
            a
        }
    } else {
        // i == j since i <= j
        i
    }
}

// O(nlogn)-time preprocessing function for Range Minimum Queries.
// It is a Sparse Table approach like the one that can be seen here: https://cp-algorithms.com/data_structures/sparse-table.html
// #[elapsed_time::elapsed]
pub fn rmq_preprocess(lcp: &[usize], s_n: usize) -> Vec<usize> {
    let lgn = flog2(s_n);

    let mut rmq_prep = vec![0; s_n * lgn];
    for i in 0..s_n {
        rmq_prep[i * lgn] = i;
    }

    let mut j = 1;
    while (1 << j) <= s_n {
        for i in 0..=s_n - (1 << j) {
            let idx_1 = i * lgn + j;
            let idx_2 = (i + (1 << (j - 1))) * lgn + j - 1;
            // This just reads:  
            // rmq_prep[idx_1] = std::cmp::min_by(
            //     rmq_prep[idx_1 - 1],
            //     rmq_prep[idx_2],
            //     |&a, &b| lcp[a].cmp(&lcp[b])
            // );
            rmq_prep[idx_1] = if lcp[rmq_prep[idx_1 - 1]] < lcp[rmq_prep[idx_2]] {
                rmq_prep[idx_1 - 1]
            } else {
                rmq_prep[idx_2]
            }
        }
        j += 1;
    }

    rmq_prep
}
