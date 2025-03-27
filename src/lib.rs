mod devices;
mod pcg;
mod rngs;

#[cfg(all(unix, feature = "std"))]
pub use devices::DevRandom;
#[cfg(all(not(unix), feature = "std"))]
pub use devices::GetRandom;
pub use devices::{FromRaw, RandomDevice};
pub use pcg::PcgXsl128_64;
pub use rngs::{RangeFromRng, Rng, ValueFromRng};

pub type SmallRng = PcgXsl128_64;
pub type DefaultRng = PcgXsl128_64;
