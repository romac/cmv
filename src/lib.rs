use std::collections::HashSet;
use std::hash::{BuildHasher, Hash};

use rand::{Rng, RngCore};

#[cfg(feature = "fxhash")]
pub type DefaultRandomState = fxhash::FxBuildHasher;

#[cfg(not(feature = "fxhash"))]
pub type DefaultRandomState = std::collections::hash_map::RandomState;

/// A Count-Min Sketch variant for approximating the count of distinct items in a stream.
///
/// See [_Distinct Elements in Streams: An Algorithm for the (Text) Book_](https://arxiv.org/pdf/2301.10191)
/// by S. **C**hakraborty, K. **M**eel, N. V. **V**inodchandran for more details about the algorithm.
///
/// # Example
///
/// ```rust
/// use rand::SeedableRng;
/// use rand::rngs::SmallRng;
///
/// use cmv::Cmv;
///
/// fn estimate_distinct(words: &[&str]) -> u128 {
///       let mut rng = SmallRng::seed_from_u64(0x123456789);
///       let mut cmv = Cmv::with_capacity(128);
///
///       for &word in words.iter() {
///           cmv.insert(word, &mut rng);
///       }
///
///       cmv.count()
/// }
/// ```
pub struct Cmv<T, S = DefaultRandomState> {
    capacity: usize,
    round: usize,
    set: HashSet<T, S>,
}

impl<T> Cmv<T, DefaultRandomState> {
    /// Create a new estimator with the given capacity and default hasher.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity,
            round: 0,
            set: HashSet::<T, DefaultRandomState>::with_capacity_and_hasher(
                capacity,
                DefaultRandomState::default(),
            ),
        }
    }
}

impl<T, S> Cmv<T, S>
where
    S: BuildHasher,
{
    /// Create a new estimator with the given capacity and hasher.
    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        Self {
            capacity,
            round: 0,
            set: HashSet::<T, S>::with_capacity_and_hasher(capacity, hasher),
        }
    }

    /// Insert an item into the estimator.
    pub fn insert(&mut self, item: T, rng: &mut dyn RngCore)
    where
        T: Eq + Hash,
    {
        if unlikely(prob_keep(rng, self.round)) {
            self.set.insert(item);
        } else {
            self.set.remove(&item);
        }

        if self.set.len() == self.capacity {
            // Remove about half of the elements
            self.set.retain(|_| unlikely(prob_keep(rng, 1)));

            // Move to next round
            self.round += 1;
        }
    }

    /// Return the current round.
    #[inline(always)]
    pub fn round(&self) -> usize {
        self.round
    }

    /// Return the sample size, i.e., the number of distinct items currently in the estimator.
    ///
    /// # Note
    /// This is not the same as the count of distinct items seen so far, as the estimator may have removed some items
    /// due to the probabilistic nature of the algorithm.
    ///
    /// See [`count`](Cmv::count) for the approximate count of distinct items.
    #[inline(always)]
    pub fn sample_size(&self) -> usize {
        self.set.len()
    }

    /// Return the approximate count of all distinct items seen so far.
    #[inline(always)]
    pub fn count(&self) -> u128 {
        let len = self.sample_size() as u128;
        len << self.round()
    }
}

#[cold]
#[inline]
fn cold() {}

#[inline]
fn unlikely(b: bool) -> bool {
    if b {
        cold()
    }
    b
}

/// Return true with probability 1/2^round
#[inline]
fn prob_keep(rng: &mut dyn RngCore, round: usize) -> bool {
    rng.random_ratio(1, 1 << round)
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    fn run<T: Eq + Hash>(capacity: usize, words: &[T], max_error: f64) {
        let distinct = words.iter().collect::<fxhash::FxHashSet<_>>();

        let mut rng = SmallRng::seed_from_u64(0x123456789);

        let mut cmv = Cmv::<&T>::with_capacity(capacity);
        for word in words {
            cmv.insert(word, &mut rng);
        }

        let diff = (cmv.count() as i128 - distinct.len() as i128).abs();
        let error = diff as f64 / distinct.len() as f64;

        println!("Exact count: {}", distinct.len());
        println!("  CMV count: {}", cmv.count());
        println!("       Diff: {diff}");
        println!("      Error: {:.2}%", error * 100.0);

        if error > max_error {
            panic!(
                "[FAILED] Error is too high: {:.2}% (max: {:.2}%)",
                error * 100.0,
                max_error * 100.0
            );
        }
    }

    #[test]
    fn hamlet_100() {
        let text = std::fs::read_to_string("hamlet.txt").unwrap();
        let words = text.split_whitespace().collect::<Vec<_>>();
        run(100, &words, 0.2);
    }

    #[test]
    fn hamlet_1000() {
        let text = std::fs::read_to_string("hamlet.txt").unwrap();
        let words = text.split_whitespace().collect::<Vec<_>>();
        run(1000, &words, 0.03);
    }

    #[test]
    fn hamlet_8000() {
        let text = std::fs::read_to_string("hamlet.txt").unwrap();
        let words = text.split_whitespace().collect::<Vec<_>>();
        run(8000, &words, 0.00);
    }

    fn gen_ints(n: u64) -> Vec<u64> {
        use rand::distr::Uniform;
        let rng = SmallRng::seed_from_u64(0x1234);

        rng.sample_iter(Uniform::new(0, n / 2).unwrap())
            .take(n as usize)
            .collect()
    }

    #[test]
    fn int_1k_100() {
        let ints = gen_ints(1_000);
        run(100, &ints, 0.05);
    }

    #[test]
    fn int_10k_1k() {
        let ints = gen_ints(10_000);
        run(1000, &ints, 0.15);
    }

    #[test]
    fn int_100k_1k() {
        let ints = gen_ints(100_000);
        run(1000, &ints, 0.15);
    }

    #[test]
    fn int_1m_1k() {
        let ints = gen_ints(1_000_000);
        run(1000, &ints, 0.05);
    }

    #[test]
    fn int_1m_10k() {
        let ints = gen_ints(1_000_000);
        run(10000, &ints, 0.02);
    }

    #[test]
    fn int_10m_10k() {
        let ints = gen_ints(10_000_000);
        run(10000, &ints, 0.01);
    }
}
