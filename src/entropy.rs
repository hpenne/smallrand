#![allow(clippy::module_name_repetitions)]

#[cfg(all(unix, feature = "std"))]
use std::fs::File;
#[cfg(all(unix, feature = "std"))]
use std::io::Read;

#[cfg(feature = "std")]
use std::collections::hash_map::RandomState;
#[cfg(feature = "std")]
use std::hash::{BuildHasher, Hasher};

/// This is a trait for entropy sources, used to produce seeds for RNGs.
pub trait EntropySource {
    /// Fills an array with random data.
    ///
    /// # Arguments
    ///
    /// * `destination`: The buffer to fill with random data
    ///
    /// # Panics
    ///
    /// May panic if the entropy source is unavailable or broken.
    ///
    fn fill(&mut self, destination: &mut [u8]);

    /// Generates an integer seed value.
    ///
    /// returns: Integer seed
    fn seed<T: FromRaw>(&mut self) -> T
    where
        Self: Sized,
    {
        T::from_raw(self)
    }
}

pub trait FromRaw {
    fn from_raw<T: EntropySource>(entropy_source: &mut T) -> Self;
}

impl FromRaw for u64 {
    fn from_raw<T: EntropySource>(entropy_source: &mut T) -> Self {
        let mut raw = [0; 8];
        entropy_source.fill(&mut raw);
        u64::from_be_bytes(raw)
    }
}

impl FromRaw for u128 {
    fn from_raw<T: EntropySource>(entropy_source: &mut T) -> Self {
        let mut raw = [0; 16];
        entropy_source.fill(&mut raw);
        u128::from_be_bytes(raw)
    }
}

/// This is an alias that maps to `DevUrandom` or `GetRandom`, depending on the platform
#[cfg(all(unix, feature = "std"))]
pub type DefaultEntropy = DevUrandom;
#[cfg(all(not(unix), feature = "allow-getrandom"))]
pub type DefaultEntropy = GetRandom;
#[cfg(all(not(unix), not(feature = "allow-getrandom"), feature = "std"))]
pub type DefaultEntropy = HashMapEntropy;

/// This is an entropy source that generates seeds by reading from /dev/urandom
#[cfg(all(unix, feature = "std"))]
pub struct DevUrandom {
    dev_random: File,
}

#[cfg(all(unix, feature = "std"))]
impl DevUrandom {
    /// Creates a new [DevUrandom] entropy source.
    ///
    /// # Panics
    ///
    /// Panics if /dev/urandom cannot be opened.
    ///
    #[must_use]
    pub fn new() -> Self {
        Self {
            dev_random: File::open("/dev/urandom").expect("Failed to open /dev/urandom"),
        }
    }
}

#[cfg(all(unix, feature = "std"))]
impl Default for DevUrandom {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(unix, feature = "std"))]
impl EntropySource for DevUrandom {
    fn fill(&mut self, destination: &mut [u8]) {
        self.dev_random
            .read_exact(destination)
            .expect("Failed to read from /dev/urandom");
        assert!(
            destination.iter().any(|v| *v != 0),
            "Entropy source generated all zeros!"
        );
    }
}

/// This is an entropy source that generates seeds using the getrandom crate.
#[cfg(all(not(unix), feature = "allow-getrandom"))]
pub struct GetRandom;

#[cfg(all(not(unix), feature = "allow-getrandom"))]
impl GetRandom {
    /// Creates a new `GetRandom` entropy source
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(all(not(unix), feature = "allow-getrandom"))]
impl Default for GetRandom {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(not(unix), feature = "allow-getrandom"))]
impl EntropySource for GetRandom {
    fn fill(&mut self, destination: &mut [u8]) {
        getrandom::fill(destination).expect("getrandom::fill failed");
        assert!(
            destination.iter().any(|v| *v != 0),
            "getrandom generated all zeros!"
        );
    }
}

/// This is an entropy source that generates seeds using std::collections::hash_map::RandomState.
/// This is likely to equivalent to ´getrandom´ on most platforms.
#[cfg(feature = "std")]
#[derive(Default)]
pub struct HashMapEntropy;

#[cfg(feature = "std")]
impl HashMapEntropy {
    /// Creates a new `HashMapEntropy` entropy source
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(feature = "std")]
impl EntropySource for HashMapEntropy {
    fn fill(&mut self, destination: &mut [u8]) {
        let mut chunks = destination.chunks_exact_mut(core::mem::size_of::<u64>());
        for chunk in chunks.by_ref() {
            let value = RandomState::new().build_hasher().finish();
            chunk.copy_from_slice(&value.to_be_bytes());
        }
    }
}

/// This implementation of `EntropySource` generates an arbitrary length output from a u64 seed
/// using the SplitMix algorithm from <https://prng.di.unimi.it/splitmix64.c>
pub struct SplitMix {
    state: u64,
}

impl SplitMix {
    /// Creates a new [SplitMix] using a u64 seed.
    ///
    /// # Arguments
    ///
    /// * `seed`: The seed value to initialize with
    ///
    /// returns: [SplitMix]
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}

impl EntropySource for SplitMix {
    fn fill(&mut self, destination: &mut [u8]) {
        let mut out_inx: usize = 0;
        while out_inx < destination.len() {
            let num = usize::min(8, destination.len() - out_inx);
            // The endianness used here should match that used in FromRaw:
            destination[out_inx..(out_inx + num)]
                .copy_from_slice(&self.next().to_be_bytes()[0..num]);
            out_inx += num;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EntropySource;

    #[cfg(all(unix, feature = "std"))]
    #[test]
    fn generate_64_bit_seed_with_dev_random() {
        let seed1: u64 = DevUrandom::new().seed();
        let seed2: u64 = DevUrandom::new().seed();
        assert_ne!(seed1, seed2);
    }

    #[cfg(all(unix, feature = "std"))]
    #[test]
    fn generate_128_bit_seed_dev_random() {
        let seed1: u64 = DevUrandom::new().seed();
        let seed2: u64 = DevUrandom::new().seed();
        assert_ne!(seed1, seed2);
    }

    #[cfg(all(not(unix), feature = "allow-getrandom"))]
    #[test]
    fn generate_64_bit_seed_with_gev_random() {
        let seed1: u64 = GetRandom::new().seed();
        let seed2: u64 = GetRandom::new().seed();
        assert_ne!(seed1, seed2);
    }

    #[test]
    fn test_splitmix_1() {
        let mut dev = SplitMix::new(42);
        let mut output = [0; 12];
        dev.fill(&mut output);
        assert_eq!(
            output,
            [189, 215, 50, 38, 47, 235, 110, 149, 40, 239, 227, 51]
        );
    }

    #[test]
    fn test_splitmix_2() {
        let mut dev = SplitMix::new(1234567);
        assert_eq!(dev.seed::<u64>(), 6457827717110365317);
        assert_eq!(dev.seed::<u64>(), 3203168211198807973);
        assert_eq!(dev.seed::<u64>(), 9817491932198370423);
        assert_eq!(dev.seed::<u64>(), 4593380528125082431);
        assert_eq!(dev.seed::<u64>(), 16408922859458223821);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_hash_map_entropy_smoke_test() {
        let mut dev = HashMapEntropy::new();
        assert_ne!(dev.seed::<u64>(), dev.seed::<u64>());
    }
}
