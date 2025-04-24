#![forbid(unsafe_code)]
extern crate core;

mod defaultrng;
mod devices;
mod ranges;
mod rngs;
mod smallrng;
mod xoshiro;

pub use defaultrng::DefaultRng;
#[cfg(all(unix, feature = "std"))]
pub use devices::DevUrandom;
#[cfg(all(not(unix), feature = "std"))]
pub use devices::GetRandom;
pub use devices::RandomDevice;
pub use rngs::Rng;
pub use smallrng::SmallRng;
pub use xoshiro::Xoshiro256pp;
