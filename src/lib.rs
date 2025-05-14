#![allow(clippy::doc_markdown)]
//! This crate provides a lightweight alternative to rand,
//! using the "xoshiro256++" (<https://prng.di.unimi.it>)
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
//! Basic usage:
//!
//! ```
//! #[cfg(feature = "std")]
//! {
//! use smallrand::StdRng;
//! let mut rng = StdRng::new();
//! let coin_flip : bool = rng.random();
//! let some_int = rng.random::<u32>();
//! let uniformly_distributed : u32 = rng.range(0..=42);
//! let a_float : f64 = rng.range(0.0..42.0);
//! }
//! ```
//!
//! `smallrand` can also be used as "no-std", in which case you can use it like this:
//! ```rust
//! use smallrand::{StdRng, SplitMix};
//! const SEED : u64 = 42;
//! let mut rng = StdRng::from_entropy(&mut SplitMix::new(SEED));
//! let some_int = rng.random::<u32>();
//! ```
//!
//! The use of the `SplitMix` may seem cumbersome, but this is done for two reasons:
//! - Requiring that the `StdRng` is initialized from something that implements `EntropySource`
//!   (like `SplitMix`) provides an interface independent of the actual algorithm used
//!   (different algorithms need different amount of entropy when initializing).
//! - Xoshiro256++ must be initialized with four u64 values. Having that as the interface could
//!   tempt users to provide only one actual value and use zero for the other three.
//!   This could cause the algorithm to produce very bad output. The use of `SplitMix`
//!   generates values that makes most algorithms perform better in this case.
//!
//! It is fairly easy to write your own implementation of `EntropySource` for your platform.
//!
#![forbid(unsafe_code)]
extern crate core;

mod chacha;
mod entropy;
mod nonces;
mod ranges;
mod rng;
mod secure_entropy;
mod smallrng;
mod stdrng;
mod xoshiro;

pub use chacha::ChaCha12;
#[cfg(feature = "std")]
pub use entropy::DefaultEntropy;
#[cfg(all(unix, feature = "std"))]
pub use entropy::DevUrandom;
pub use entropy::EntropySource;
#[cfg(all(not(unix), feature = "std"))]
pub use entropy::GetRandom;
#[cfg(feature = "std")]
pub use entropy::HashMapEntropy;
pub use entropy::SplitMix;
pub use rng::Rng;
#[cfg(feature = "std")]
pub use secure_entropy::SecureEntropy;
pub use smallrng::SmallRng;
pub use stdrng::StdRng;
pub use xoshiro::Xoshiro256pp;
