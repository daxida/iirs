//! Bridge between the RMQ structures and the kangaroo step.

use rmq::Rmq;

/// An RMQ over the LCP array, paired with the array itself.
///
/// The Rmq structures report the *index* of a minimum, but the kangaroo step wants *value*.
pub struct LcpMin<'a, R: Rmq> {
    rmq: R,
    lcp: &'a [u32],
}

impl<'a, R: Rmq> LcpMin<'a, R> {
    pub fn new(rmq: R, lcp: &'a [u32]) -> Self {
        Self { rmq, lcp }
    }

    /// The smallest LCP value in `[i, j)`, or zero if the range is empty.
    #[inline(always)]
    pub fn min(&self, i: usize, j: usize) -> usize {
        self.rmq.rmq(i, j).map_or(0, |k| self.lcp[k] as usize)
    }
}
