mod devices;
mod pcg;
mod rngs;

#[cfg(all(unix, feature = "std"))]
pub use devices::DevRandom;
#[cfg(all(not(unix), feature = "std"))]
pub use devices::GetRandom;
pub use devices::RandomDevice;
pub use pcg::PcgXsl128_64;
pub use rngs::Rng;

/// This an alias to a Numerically good PRNG if you need something small and fast
/// but not cryptographically secure.
/// The algorithm may change at any time, so if your
/// code depends on the algorithm staying the same then you should
/// use a specific algorithm instead.
pub type SmallRng = PcgXsl128_64;

/// This is the type alias for the default PRNG.
/// It is currently not cryptographically secure, but if such an algorithm
/// is added later, then it will be used as the DefaultRng.
/// The algorithm may change at any time, so if your
/// code depends on the algorithm staying the same then you should
/// use a specific algorithm instead.
pub type DefaultRng = PcgXsl128_64;
