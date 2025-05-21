use std::hash::Hash;
use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

use cmv::Cmv;
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

pub fn hamlet(c: &mut Criterion) {
    let words = std::fs::read_to_string("hamlet.txt").unwrap();
    let words = words.split_whitespace().collect::<Vec<_>>();

    c.bench_function("hamlet/7806/100", |b| {
        b.iter(|| run_bench(&words, black_box(100)))
    });
    c.bench_function("hamlet/7806/1k", |b| {
        b.iter(|| run_bench(&words, black_box(1_000)))
    });
}

pub fn ints(c: &mut Criterion) {
    use rand::SeedableRng;

    let mut rng = SmallRng::seed_from_u64(0x1234);

    c.bench_function("ints/10k/1k", |b| {
        let ints = gen_ints(1_000, &mut rng);
        b.iter(|| run_bench(&ints, black_box(1_000)))
    });

    c.bench_function("ints/100k/1k", |b| {
        let ints = gen_ints(100_000, &mut rng);
        b.iter(|| run_bench(&ints, black_box(1_000)))
    });

    c.bench_function("ints/1m/1k", |b| {
        let ints = gen_ints(1_000_000, &mut rng);
        b.iter(|| run_bench(&ints, black_box(1_000)))
    });

    c.bench_function("ints/1m/10k", |b| {
        let ints = gen_ints(1_000_000, &mut rng);
        b.iter(|| run_bench(&ints, black_box(10_000)))
    });
}

fn gen_ints<R: rand::Rng>(n: u64, rng: R) -> Vec<u64> {
    use rand::distr::Uniform;

    rng.sample_iter(Uniform::new(0, n / 2).unwrap())
        .take(n as usize)
        .collect()
}

criterion_group!(benches, ints, hamlet);
criterion_main!(benches);
