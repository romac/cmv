# cmv

![Crates.io Version](https://img.shields.io/crates/v/cmv)
![Crates.io License](https://img.shields.io/crates/l/cmv)
![Crates.io MSRV](https://img.shields.io/crates/msrv/cmv)

Rust implementation of [_Distinct Elements in Streams: An Algorithm for the (Text) Book_](https://arxiv.org/pdf/2301.10191) by S. **C**hakraborty, K. **M**eel, N. V. **V**inodchandran.

## Usage

```rust
use rand::SeedableRng;
use rand::rngs::SmallRng;

use cmv::Cmv;

fn estimate_distinct(words: &[&str]) -> u128 {
      let mut rng = SmallRng::seed_from_u64(0x123456789);
      let mut cmv = Cmv::with_capacity(128);

      for &word in words.iter() {
          cmv.insert(word, &mut rng);
      }

      cmv.count()
}
```

## License

Licensed under the [Apache 2.0 license](./LICENSE.md).
