smallrand
=========

[![Test Status](https://github.com/hpenne/smallrand/actions/workflows/rust.yml/badge.svg?event=push)](https://github.com/hpenne/smallrand/actions)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

Random number generation with absolutely minimal dependencies and no unsafe code.

This crate provides a lightweight alternative to [`rand`](https://crates.io/crates/rand).
It implements the same two algorithms as `rand`'s `SmallRng` and `StdRng` (Xoshiro256++ and ChaCha12),
using the same aliases,
and provides all basic functions you expect including uniformly distributed integers and floats in a user-specified
range.
Those who are sometimes frustrated by `rand`'s API might prefer `smallrand`'s API.

The crate is intended to be easy to audit.
It is small and uses no unsafe code.
Its only dependency is [`getrandom`](https://crates.io/crates/getrandom), and that is only used on non-Linux/Unix
platforms.

It can also be built as no-std, in which case you'll have to provide your own seeds.

Quick start
-----------

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
    - By default, the seed is read from /dev/urandom on Linux-like platforms, and comes from the `getrandom` crate for
      others.
      You can also implement your own `EntropySource` and use that to provide the seed.
* Why don't you get the seeds from `hash_map::RandomState` like `fastrand` does and remove the dependency on
  `getrandom`?
    - `RandomState` provides 128 bits of entropy at best, and only reads that from the system's entropy source at
      startup. It then uses a no-secure algorithm to derive more seeds from that. This is not good enough as a default
      for everyone. However, you can opt out of depending on `getrandom` by building without the `allow-getrandom`
      feature flag, in which case `RandomState` will be used. Note that `/dev/urandom` is always used on Unix-like
      platforms.
* Why would I choose this over `rand`?
    - `rand` is large and difficult to audit. Its dependencies (as of version 0.9) include `zerocopy`,
      which contains a huge amount of unsafe code.
    - Its API encourages you to use thread local RNG instances. This creates unnecessary (thread) global state,
      which is almost always a bad idea.
      Since it is thread local, you also get one RNG per thread in the thread pool if your
      code is async.
    - Unlike `rand`, `smallrand` crate does not require you to import any traits or anything else beyond the RNG you're
      using.
    - This crate has minimal dependencies and does not intend to change much, so you won't have to update it very often.
    - This crate compiles faster than `rand` due to it smaller size and minimal dependencies.
* Why would I choose this over `fastrand`?
    - `fastrand` uses Wyrand as its algorithm, which does not seem to be as respected as Xoshiro256++ (`SmallRng`).
    - `smallrand` also offers ChaCha12 (`StdRng`), which has much better security properties than Wyrand.
    - Just like `rand` its API encourages you to use thread local RNG instances.
    - `fastrand` gets its entropy from `std::collections::hash_map::RandomState`.
      This provides somewhat limited entropy (see above), although perhaps enough to initialize Wyrand given its smaller
      state.
* How fast is this compared to `rand`?
    - `smallrand` seems to be slightly faster overall on a Apple M1 (see below).
* Is the `StdRng` cryptographically secure?
    - Just as with StdRng in `rand` it might be (depending on how you define the term), but this not in any way
      guaranteed.
      See also the next section.
* Can this be used "no-std"?
    - Yes, please see the crate documentation for an example.

Security
--------

`StdRng` uses the ChaCha crypto algorithm with 12 rounds.
Current thinking seems to be that 8 rounds is sufficient ([Too Much Crypto](https://eprint.iacr.org/2019/1492.pdf)),
but 12 currently used for extra security margin.
This algorithm is well respected and is currently unbroken, and is as such not predictable.
It can likely be used to implement random generators that are cryptographically secure in practice,
but please note that no guarantees of any kind are made that this particular implementation is cryptographically secure.

Also note that for a random generator implementation to be certifiable as cryptographically secure,
it needs to be implemented according to NIST SP 800-90A.
ChaCha is not one of the approved algorithms allowed by NIST SP 800-90A.

`SmallRng` uses Xoshiro256++ which is a predictable RNG.
An attacker that is able to observe its output will be able to calculate its internal state and predict its output,
which means that it is not cryptographically secure.
It has this in common with other algorithms of similar size and complexity, like PCG and Wyrand.

`smallrand` makes a modest effort to detect fatal failures of the entropy source when creating an `StdRng` with `new()`,
including the Health Tests of NIST SP 800-90B.

Speed
-----

`SmallRng` from `smallrand` has been benchmarked against `SmallRng` from the `rand` crate (which uses the same
algorithm) using  `criterion`.
On my Apple M1, `smallrand` is equal in performance when generating u64 values, more than twice as fast generating
uniformly distributed ranges of u64 values,
and approximately 10% faster when filling a slice of bytes with random data.

`rand` is 7% faster at generating ranges of f64 values, which could be caused by `rand` using a slightly simpler
algorithm which does not use the full available dynamic range of the mantissa when the generated value is close to zero.

`StdRng` from `smallrand` has been similarly benchmarked, and was approximately 4% faster than the same algorithm from
`rand` when generating u64 values.

`smallrand` has not been benchmarked against `fastrand`, but `fastrand` is expected to be faster as it uses an algorithm
with smaller state.
