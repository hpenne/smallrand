smallrand
=========

Random number generation with absolutely minimal dependencies and no unsafe code.

This crate provides a lightweight alternative to [`rand`](https://crates.io/crates/rand), based on the
PCG (<https://www.pcg-random.org>) random generator algorithm (which is also implemented by `rand`).

The crate is intended to be easy to audit. Its only dependency is [`getrandom`](https://crates.io/crates/getrandom), and
that is only used on non-Linux platforms. It can also be built as non-std, in which case `getrandom` is not used at
all (but you´ll have to provide the seed yourself).

Quick start
-----

```toml
use smallrand;
let mut rng = DefaultRng::new();
let coin_flip : bool = rng.random();
let some_int = rng.random::<u32>();
let uniformly_distributed : u32 = rng.range(0.. = 42);
```

FAQ
---

* Why would I choose this over `rand`?
    - `rand` is very large and difficult to audit. It's dependencies (as of 0.9) include `zerocopy`, which contains a
      huge amount of
      unsafe code.
    - Its API encourages you to use `thread_rng` to create RNGs. This creates unnecessary global state, which is almost
      always a bad idea. Since it is thread local, you also get one RNG per thread in the thread pool if your code is
      async. Furthermore, it is a potential security risk (see [below](#the-juniper-incident)).
    - `smallrand` does not require you to import any traits or anything else beyond the RNG you're using.

## The Juniper incident

