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
