smallrand
=========

[![Test Status](https://github.com/hpenne/smallrand/actions/workflows/rust.yml/badge.svg?event=push)](https://github.com/hpenne/smallrand/actions)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

Random number generation with absolutely minimal dependencies and no unsafe code.

This crate provides a lightweight alternative to [`rand`](https://crates.io/crates/rand), using the "
xoshiro256++" (<https://prng.di.unimi.it>) and "ChaCha12"
algorithms (https://cr.yp.to/chacha.html), which are also the ones used by `rand` for its `SmallRng` and `StdRng`,
respectively.

The crate is intended to be easy to audit. Its only dependency is [`getrandom`](https://crates.io/crates/getrandom), and
that is only used on non-Linux/Unix platforms. It can also be built as no-std, in which case `getrandom` is not used at
all (but you´ll then have to provide the seed yourself).

Quick start
-----

```rust
use smallrand::StdRng;
let mut rng = StdRng::new();
let coin_flip : bool = rng.random();
let some_int = rng.random::<u32>();
let uniformly_distributed : u32 = rng.range(0..=42);
let a_float : f64 = rng.range(0.0..42.0);
```

FAQ
---

* Where does the seed come from?
    - The seed is read from /dev/urandom on Linux-like platforms, and comes from the `getrandom` crate for others. You
      can also write your own `RandomDevice` and use that to provide the seed.
* Is the DefaultRng cryptographically secure?
    - The `DefaultRng` uses the ChaCha12 crypto algorithm. This algorithm is currently unbroken and can be used to
      implement cryptographically secure random generators, but please note that no guarantees of any kind are made that
      this particular implementation is cryptographically secure.
* How fast is this compared to `rand`?
    - `SmallRng` from `smallrand` has been benchmarked against the `rand` crate (`SmallRng`/Xoshiro256++) using
      `criterion`. On my Apple M1, `smallrand` is equal in performance when generating u64 values, more than twice as
      fast generating uniformly distributed ranges
      of u64 values, and approximately 10% faster when filling a slice of bytes with random data. `rand` is 7% faster at
      generating ranges of f64 values, which could be caused by `rand` using a slightly simpler algorithm which does not
      use the full available dynamic range of the mantissa when the generated value is close to zero.
    - `StdRng` from `smallrand` has been similarly benchmarked, and was approximately 4% faster than the same algorithm
      from `rand` when generating u64 values.
* Why would I choose this over `rand`?
    - `rand` is large and difficult to audit. Its dependencies (as of version 0.9) include `zerocopy`, which contains a
      huge amount of unsafe code.
    - Its API encourages you to use thread local RNG instances. This creates unnecessary (thread) global state, which is
      almost always a bad idea. Since it is thread local, you also get one RNG per thread in the thread pool if your
      code is async.
    - Unlike `rand`, this crate does not require you to import any traits or anything else beyond the RNG you're using.
    - This crate has minimal dependencies and does not intend to change much, so you won't have to update it very often.
    - This crate compiles faster than `rand` due to it smaller size and minimal dependencies.
* Why would I choose this over `fastrand`?
    - `fastrand` uses Wyrand as its algorithm, which does not seem to be as respected as ChaCha12 and Xoshiro256++.
    - Just like `rand` its API encourages you to use thread local RNG instances.
