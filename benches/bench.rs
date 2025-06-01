use std::hash::Hash;

use cmv::Cmv;
use divan::{bench, Bencher};
use rand::rngs::SmallRng;
use rand::SeedableRng;

fn run_bench<T>(words: &[T], capacity: usize) -> u128
where
    T: Eq + Hash,
{
    let mut rng = SmallRng::seed_from_u64(0x1234);
    let mut cmv = Cmv::<&T>::with_capacity(capacity);
    for word in words {
        cmv.insert(word, &mut rng);
    }
    cmv.count()
}

#[bench(args = [100, 1_000, 10_000])]
pub fn hamlet(b: Bencher, capacity: usize) {
    let words = std::fs::read_to_string("hamlet.txt").unwrap();

    b.with_inputs(|| words.split_whitespace().collect::<Vec<_>>())
        .input_counter(|words| words.len())
        .bench_refs(|words| run_bench(words, capacity));
}

const SEED: u64 = 0x1234;

fn gen_ints<R: rand::Rng>(n: u64, rng: R) -> Vec<u64> {
    use rand::distr::Uniform;

    rng.sample_iter(Uniform::new(0, n / 2).unwrap())
        .take(n as usize)
        .collect()
}

#[bench(args = [(10_000, 1_000), (100_000, 1_000), (1_000_000, 1_000), (1_000_000, 10_000)])]
pub fn ints(b: Bencher, (count, capacity): (u64, usize)) {
    use rand::SeedableRng;

    b.with_inputs(|| {
        let mut rng = SmallRng::seed_from_u64(SEED);
        gen_ints(count, &mut rng)
    })
    .input_counter(|ints| ints.len())
    .bench_refs(|ints| run_bench(ints, capacity));
}

fn main() {
    divan::main();
}
