#[inline(always)]
fn flog2(v: usize) -> usize {
    v.ilog2() as usize
}

pub trait Rmq {
    fn rmq(&self, i: usize, j: usize) -> usize;
}

pub struct Sparse {
    lgn: usize,
    table: Vec<usize>,
}

impl Sparse {
    pub fn new(array: &[usize]) -> Self {
        let n = array.len();
        let lgn = flog2(n) + 1;

        let mut table: Vec<usize> = vec![0; lgn * n];

        for j in 0..n {
            table[j * lgn] = array[j];
        }

        for i in 1..lgn {
            for j in 0..=(n - (1 << i)) {
                table[j * lgn + i] = std::cmp::min(
                    table[j * lgn + i - 1],
                    table[(j + (1 << (i - 1))) * lgn + i - 1],
                );
            }
        }

        Sparse { lgn, table }
    }
}

impl Rmq for Sparse {
    fn rmq(&self, i: usize, j: usize) -> usize {
        debug_assert!(i < j);

        let k = flog2(j - i + 1);
        std::cmp::min(
            self.table[i * self.lgn + k],
            self.table[(j + 1 - (1 << k)) * self.lgn + k],
        )
    }
}
