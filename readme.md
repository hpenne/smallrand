smallrand
=========

[![Test Status](https://github.com/hpenne/smallrand/actions/workflows/rust.yml/badge.svg?event=push)](https://github.com/hpenne/smallrand/actions)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

Random number generation with absolutely minimal dependencies and no unsafe code.

This crate provides a lightweight alternative to [`rand`](https://crates.io/crates/rand), using the "xoshiro256++"
algorithm (<https://prng.di.unimi.it>), which is the one used by `rand` for its `SmallRng`.

The crate is intended to be easy to audit. Its only dependency is [`getrandom`](https://crates.io/crates/getrandom), and
that is only used on non-Linux/Unix platforms. It can also be built as no-std, in which case `getrandom` is not used at
all (but you´ll then have to provide the seed yourself).

Quick start
-----

```rust
use smallrand;
let mut rng = DefaultRng::new();
let coin_flip : bool = rng.random();
let some_int = rng.random::<u32>();
let uniformly_distributed : u32 = rng.range(0..=42);
```

FAQ
---

* Where does the seed come from?
    - The seed is read from /dev/random on Linux-like platform, and comes from the `getrandom` crate for others.
* Why is there no CSPRNG?
    - First, don't implement crypto yourself. If you think that is a good idea, then you probably don't know the subject
      well enough.
    - If you have reached the level where you are competent enough to know you should not and have then gone beyond
      that, then you probably will want implement the CSPRNG yourself.
* How fast is this compared to `rand` or `fastrand`
* Why would I choose this over `rand`?
    - `rand` is very large and difficult to audit. Its dependencies (as of 0.9) include `zerocopy`, which contains a
      huge amount of unsafe code.
    - Its API encourages you to use thread local RNG instances. This creates unnecessary (thread) global state, which is
      almost always a bad idea. Since it is thread local, you also get one RNG per thread in the thread pool if your code is
      async. Furthermore, it is a potential security risk (see [below](#the-juniper-incident)).
    - Unlike `rand`, this crate does not require you to import any traits or anything else beyond the RNG you're using.
    - This crate has minimal dependencies and does not intend to change much, so you won't have to update it very often.
    - This crate compiles much faster than `rand`.
* Why would I choose this over `fastrand`?
    - `fastrand` uses Wyrand as its algorithm, which does not seem to be as respected as Xoshiro256++.
    - When you use `fastrand` to generate integers in a range, it does not generate a uniform distribution (as of March
      30th 2025). The code uses a simple modulus, which is plain wrong. This call the quality of the whole crate into
      question.

## The Juniper incident

