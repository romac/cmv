use std::hash::Hash;

use rand::{Rng, RngCore};

type Set<T> = fxhash::FxHashSet<T>;
// type Set<T> = hashbrown::HashSet<T>;
// type Set<T> = ahash::AHashSet<T>;

pub struct Cmv<T> {
    capacity: usize,
    round: usize,
    set: Set<T>,
    rng: Box<dyn RngCore>,
}

impl<T> Cmv<T> {
    pub fn new(capacity: usize, rng: impl RngCore + 'static) -> Self {
        Self {
            capacity,
            round: 0,
            set: Set::with_capacity_and_hasher(capacity, fxhash::FxBuildHasher::default()),
            rng: Box::new(rng),
        }
    }

    pub fn insert(&mut self, item: T)
    where
        T: Eq + Hash,
    {
        if prob_keep(&mut self.rng, self.round) {
            self.set.insert(item);
        } else {
            self.set.remove(&item);
        }

        if self.set.len() == self.capacity {
            // Remove about halft of the elements
            self.set.retain(|_| prob_keep(&mut self.rng, 1));

            // Move to next round
            self.round += 1;
        }
    }

    #[inline(always)]
    pub fn round(&self) -> usize {
        self.round
    }

    #[inline(always)]
    pub fn sample_size(&self) -> usize {
        self.set.len()
    }

    /// Return the approximate count of distinct items
    #[inline(always)]
    pub fn count(&self) -> u128 {
        let len = self.sample_size() as u128;
        len << self.round()
    }
}

/// Return true with probablity 1/2^round
#[cold]
#[inline(always)]
fn prob_keep(rng: &mut dyn RngCore, round: usize) -> bool {
    rng.gen_ratio(1, 1 << round)
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::SeedableRng;
    use rand_chacha::ChaChaRng;

    fn run<T: Eq + Hash>(capacity: usize, words: &[T]) {
        let distinct = words.iter().collect::<Set<_>>();

        let rng = ChaChaRng::seed_from_u64(0x1234);

        let mut cmv = Cmv::new(capacity, rng);
        for word in words {
            cmv.insert(word);
        }

        let diff = (cmv.count() as i128 - distinct.len() as i128).abs();
        let error = (diff as f64 / distinct.len() as f64) * 100.0;

        println!("Exact count: {}", distinct.len());
        println!("  CMV count: {}", cmv.count());
        println!("       Diff: {}", diff);
        println!("      Error: {:.2}%", error);
    }

    #[test]
    fn hamlet_100() {
        let text = std::fs::read_to_string("hamlet.txt").unwrap();
        let words = text.split_whitespace().collect::<Vec<_>>();
        run(100, &words);
    }

    #[test]
    fn hamlet_1000() {
        let text = std::fs::read_to_string("hamlet.txt").unwrap();
        let words = text.split_whitespace().collect::<Vec<_>>();
        run(1000, &words);
    }

    #[test]
    fn hamlet_8000() {
        let text = std::fs::read_to_string("hamlet.txt").unwrap();
        let words = text.split_whitespace().collect::<Vec<_>>();
        run(8000, &words);
    }

    fn gen_ints(n: u64) -> Vec<u64> {
        use rand::distributions::Uniform;
        let rng = ChaChaRng::seed_from_u64(0x1234);

        rng.sample_iter(Uniform::new(0, n / 2))
            .take(n as usize)
            .collect()
    }

    #[test]
    fn int_1k_100() {
        let ints = gen_ints(1_000);
        run(100, &ints);
    }

    #[test]
    fn int_10k_1k() {
        let ints = gen_ints(10_000);
        run(1000, &ints);
    }

    #[test]
    fn int_100k_1k() {
        let ints = gen_ints(100_000);
        run(1000, &ints);
    }

    #[test]
    fn int_1m_1k() {
        let ints = gen_ints(1_000_000);
        run(1000, &ints);
    }

    #[test]
    fn int_1m_10k() {
        let ints = gen_ints(1_000_000);
        run(10000, &ints);
    }

    #[test]
    fn int_10m_10k() {
        let ints = gen_ints(10_000_000);
        run(10000, &ints);
    }
}
