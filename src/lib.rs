use std::hash::Hash;

use fxhash::FxHashSet;
use rand::{Rng, RngCore};

pub struct Cmv<T> {
    capacity: usize,
    round: usize,
    set: FxHashSet<T>,
    rng: Box<dyn RngCore>,
}

impl<T> Cmv<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            round: 0,
            set: FxHashSet::default(),
            rng: Box::new(rand::thread_rng()),
        }
    }

    pub fn insert(&mut self, item: T)
    where
        T: Eq + Hash,
    {
        self.set.remove(&item);

        if prob_keep(&mut self.rng, self.round) {
            self.set.insert(item);
        }

        if self.set.len() == self.capacity {
            self.set.retain(|_| prob_keep(&mut self.rng, 1));
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
#[inline(always)]
fn prob_keep(rng: &mut dyn RngCore, round: usize) -> bool {
    rng.gen_bool(1.0_f64 / 2.0_f64.powi(round as i32))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run<T: Eq + Hash>(capacity: usize, words: &[T]) {
        let distinct = words.iter().collect::<FxHashSet<_>>();

        let mut cmv = Cmv::new(capacity);
        for word in words {
            cmv.insert(word);
        }

        let diff = (cmv.count() as i128 - distinct.len() as i128).abs();

        println!("Exact count: {}", distinct.len());
        println!("  CMV count: {}", cmv.count());
        println!("       Diff: {}", diff);
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

    fn gen_ints<R: rand::Rng>(n: u64, rng: R) -> Vec<u64> {
        use rand::distributions::Uniform;
        rng.sample_iter(Uniform::new(0, n / 2))
            .take(n as usize)
            .collect()
    }

    #[test]
    fn int_1k() {
        let ints = gen_ints(1_000, rand::thread_rng());
        run(100, &ints);
    }

    #[test]
    fn int_10k() {
        let ints = gen_ints(10_000, rand::thread_rng());
        run(1000, &ints);
    }

    #[test]
    fn int_100k() {
        let ints = gen_ints(100_000, rand::thread_rng());
        run(1000, &ints);
    }

    #[test]
    fn int_1m() {
        let ints = gen_ints(1_000_000, rand::thread_rng());
        run(1000, &ints);
    }
}
