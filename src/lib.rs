//! This crate provides a lightweight alternative to rand,
//! using the " xoshiro256++" (<https://prng.di.unimi.it>)
//! and "ChaCha12" algorithms (<https://cr.yp.to/chacha.html>),
//! which are also the ones used by [`rand`](https://crates.io/crates/rand) for its SmallRng
//! and StdRng, respectively.
//!
//! The crate is intended to be easy to audit.
//! Its only dependency is [`getrandom`](https://crates.io/crates/getrandom),
//! and that is only used on non-Linux/Unix platforms.
//! It can also be built as no-std, in which case [`getrandom`](https://crates.io/crates/getrandom)
//! is not used at all (but you´ll then have to provide the seed yourself).
//!
//! Quick start
//!
//! ```
//! use smallrand::StdRng;
//! let mut rng = StdRng::new();
//! let coin_flip : bool = rng.random();
//! let some_int = rng.random::<u32>();
//! let uniformly_distributed : u32 = rng.range(0..=42);
//! let a_float : f64 = rng.range(0.0..42.0);
//! ```
#![forbid(unsafe_code)]
extern crate core;

mod chacha;
mod devices;
mod ranges;
mod rng;
mod smallrng;
mod stdrng;
mod xoshiro;

pub use chacha::ChaCha12;
#[cfg(all(unix, feature = "std"))]
pub use devices::DevUrandom;
#[cfg(all(not(unix), feature = "std"))]
pub use devices::GetRandom;
pub use devices::RandomDevice;
pub use rng::Rng;
pub use smallrng::SmallRng;
pub use stdrng::StdRng;
pub use xoshiro::Xoshiro256pp;
