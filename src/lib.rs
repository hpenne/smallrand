#![forbid(unsafe_code)]
extern crate core;

mod devices;
mod ranges;
mod rngs;
mod smallrng;
mod xoshiro;

#[cfg(all(unix, feature = "std"))]
pub use devices::DevUrandom;
#[cfg(all(not(unix), feature = "std"))]
pub use devices::GetRandom;
pub use devices::RandomDevice;
pub use rngs::Rng;
pub use smallrng::SmallRng;
pub use xoshiro::Xoshiro256pp;

/// This is the type alias for the default PRNG.
/// It is currently not cryptographically secure, but if such an algorithm
/// is added later, it will be used as the `DefaultRng`.
/// The algorithm may change at any time, so if your
/// code depends on the algorithm staying the same then you should
/// use a specific algorithm instead.
pub type DefaultRng = Xoshiro256pp;
