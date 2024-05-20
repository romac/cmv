use std::hash::Hash;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use cmv::Cmv;

fn run_bench<T>(words: &[T], capacity: usize) -> u128
where
    T: Eq + Hash,
{
    let mut cmv = Cmv::new(capacity);
    for word in words {
        cmv.insert(word);
    }
    cmv.count()
}

pub fn hamlet(c: &mut Criterion) {
    let words = std::fs::read_to_string("hamlet.txt").unwrap();
    let words = words.split_whitespace().collect::<Vec<_>>();

    c.bench_function("hamlet/7806/100", |b| {
        b.iter(|| run_bench(&words, black_box(100)))
    });
    c.bench_function("hamlet/7806/1000", |b| {
        b.iter(|| run_bench(&words, black_box(1000)))
    });
    c.bench_function("hamlet/7806/8000", |b| {
        b.iter(|| run_bench(&words, black_box(8000)))
    });
}

pub fn ints(c: &mut Criterion) {
    use rand::SeedableRng;

    let mut rng = rand_chacha::ChaChaRng::seed_from_u64(0x1234);

    c.bench_function("ints/10k/1000", |b| {
        let ints = gen_ints(1_000, &mut rng);
        b.iter(|| run_bench(&ints, black_box(1000)))
    });

    c.bench_function("ints/100k/1000", |b| {
        let ints = gen_ints(100_000, &mut rng);
        b.iter(|| run_bench(&ints, black_box(1000)))
    });

    c.bench_function("ints/1m/1000", |b| {
        let ints = gen_ints(1_000_000, &mut rng);
        b.iter(|| run_bench(&ints, black_box(1000)))
    });
}

fn gen_ints<R: rand::Rng>(n: u64, rng: R) -> Vec<u64> {
    use rand::distributions::Uniform;

    rng.sample_iter(Uniform::new(0, n / 2))
        .take(n as usize)
        .collect()
}

criterion_group!(benches, ints);
criterion_main!(benches);
