use std::cmp::Ordering;

#[inline(always)]
fn flog2(v: usize) -> usize {
    v.ilog2() as usize
}

// Range Minimum Query (used in algo::lce)
pub fn rmq(rmq_prep: &[usize], lcp: &[usize], s_n: usize, mut i: usize, mut j: usize) -> usize {
    let lgn = flog2(s_n);

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
            let a = rmq_prep[i * lgn + k];
            let b = rmq_prep[(j - (1 << k) + 1) * lgn + k];

            if lcp[a] > lcp[b] {
                b
            } else {
                a
            }
        }
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
