# smallrand

[![Test Status](https://github.com/hpenne/smallrand/actions/workflows/rust.yml/badge.svg?event=push)](https://github.com/hpenne/smallrand/actions)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

Random number generation with absolutely minimal dependencies and no unsafe code.

This crate provides a lightweight alternative to [`rand`](https://crates.io/crates/rand).
It implements the same two algorithms as `rand`'s `SmallRng` and `StdRng` (Xoshiro256++ and ChaCha12),
using the same aliases,
and provides all basic functions you expect including uniformly distributed integers and floats in a user-specified range.
Those who are sometimes frustrated by `rand`'s API might prefer `smallrand`'s API.

The crate is intended to be easy to audit.
It is small and uses no unsafe code.
Its only dependency is [`getrandom`](https://crates.io/crates/getrandom), and that is only used on non-Linux/Unix
platforms.

It can also be built as no-std, in which case you'll have to provide your own seeds.

## Quick start

```rust
use smallrand::StdRng;
let mut rng = StdRng::new();
let coin_flip : bool = rng.random();
let some_int = rng.random::<u32>();
let uniformly_distributed : u32 = rng.range(0..=42);
let a_float : f64 = rng.range(0.0..42.0);
```

## FAQ

* Where does the seed come from?
    - By default, the seed is read from /dev/urandom on Linux-like platforms, and comes from the `getrandom` crate for
      others.
      You can also implement your own `EntropySource` and use that to provide the seed.
* Why don't you get the seeds from `hash_map::RandomState` like `fastrand` does and remove the dependency on
  `getrandom`?
    - `RandomState` reads 128 bits of entropy from the system's entropy source at startup.
      It then uses a non-secure algorithm to derive more seeds from that.
      This provides limited entropy and is not good enough as a default for everyone.
      However, you can opt out of depending on `getrandom` by building without the `allow-getrandom` feature flag,
      in which case `RandomState` _will_ be used.
      Note that `/dev/urandom` is always used on Unix-like platforms, regardless.
* Why would I choose this over `rand`?
    - `rand` is large and difficult to audit. Its dependencies (as of version 0.9) include `zerocopy`,
      which contains a huge amount of unsafe code.
    - Its API encourages you to use thread local RNG instances. This creates unnecessary (thread) global state,
      which is almost always a bad idea.
      Since it is thread local, you also get one RNG per thread in the thread pool if your code is async.
    - Unlike `rand`, `smallrand` crate does not require you to import any traits or anything else beyond the RNG you're
      using.
    - This crate has minimal dependencies and does not intend to change much, so you won't have to update it very often.
    - This crate compiles faster than `rand` due to its smaller size and minimal dependencies.
* Why would I choose this over `fastrand`?
    - If you think the algorithms used are preferable to Wyrand.
    - `fastrand` gets its entropy from `std::collections::hash_map::RandomState`.
      This provides somewhat limited entropy (see above), although perhaps enough to initialize Wyrand given its smaller
      state.
    - Just like `rand` its API encourages you to use thread local RNG instances.
* How fast is this compared to `rand`?
    - `smallrand` seems to be slightly faster overall on a Apple M1 (see [Speed](#speed) below).
* Is the `StdRng` cryptographically secure?
    - Just as with `StdRng` in `rand` it might be (depending on how you define the term), but this not in any way guaranteed.
      See also the next section.
* Can this be used "no-std"?
    - Yes, please see the crate documentation for an example.

## Security

`StdRng` uses the ChaCha crypto algorithm with 12 rounds.
Current thinking seems to be that 8 rounds is sufficient ([Too Much Crypto](https://eprint.iacr.org/2019/1492.pdf)),
but 12 is currently used for extra security margin.
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

## Speed

`smallrand` has been benchmarked against the v.0.9 of the `rand` crate using  `criterion` on a MacBook Air M1:

*Algorithm* | *Operation*    | *`rand`* | *`smallrand`*
:---------------|:---------------|---------:|-----:
SmallRng (Xoshiro256++) | generate u64   |  1.145ns |  1.141ns
SmallRng (Xoshiro256++) | fill 256 bytes |  38.66ns | 35.99ns
SmallRng (Xoshiro256++) | range (u64)    |   3.84ns | 1.46ns
SmallRng (Xoshiro256++) | range (f64)    |   1.17ns | 1.24ns
StdRng (Chacha 12) | fill 256 bytes |  254.8ns | 233.1ns
StdRng (Chacha 12) | generate u64   |   8.64ns | 7.32ns

In these benchmarks, `smallrand` is a little faster overall than `rand` on this platform,
although `rand` is a little faster at generating uniformly distributed f64 in a specified range.
This could be because `smallrand` uses a different algorithm that uses the full dynamic range of the mantissa even for very small values (`rand` does not).
On the other hand, `smallrand` is 2.6 times faster then `rand` at generating uniformly distributed u64 in a specified range (using `SmallRng`).
