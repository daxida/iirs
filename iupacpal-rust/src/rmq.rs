#[inline(always)]
fn flog2(v: usize) -> usize {
    v.ilog2() as usize
}

// Range Minimum Query (used in algo::lce)
pub fn rmq(rmq_prep: &[usize], lcp: &[usize], s_n: usize, mut i: usize, mut j: usize) -> usize {
    // We could pass this as an arg to prevent recomputation but it's not worth.
    let lgn = flog2(s_n);

    assert!(i != j);

    if i > j {
        std::mem::swap(&mut i, &mut j);
    }

    i += 1;
    assert!(i <= j);

    if i < j {
        let k = flog2(j - i + 1);
        let a = rmq_prep[i * lgn + k];
        let b = rmq_prep[(j - (1 << k) + 1) * lgn + k];

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

// O(nlogn)-time preprocessing function for Range Minimum Queries (used in main::main)
#[elapsed_time::elapsed]
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
