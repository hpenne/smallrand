#![forbid(unsafe_code)]

#[cfg(all(unix, feature = "std"))]
use std::fs::File;
#[cfg(all(unix, feature = "std"))]
use std::io::Read;

/// This is a trait for random devices.
/// Random devices are random sources used to produce seeds for RNGs.
pub trait RandomDevice {
    /// Generates an array of random bytes.
    ///
    /// returns: Array of random u8 values
    fn seed_bytes<const N: usize>(&mut self) -> [u8; N];

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
    fn from_raw<T: RandomDevice>(device: &mut T) -> Self;
}

impl FromRaw for u64 {
    fn from_raw<T: RandomDevice>(device: &mut T) -> Self {
        u64::from_be_bytes(device.seed_bytes::<8>())
    }
}

impl FromRaw for u128 {
    fn from_raw<T: RandomDevice>(device: &mut T) -> Self {
        u128::from_be_bytes(device.seed_bytes::<16>())
    }
}

/// This is a random device that maps to DevUrandom or GetRandom, depending on the platform
#[cfg(all(unix, feature = "std"))]
pub type DefaultDevice = DevUrandom;
#[cfg(all(not(unix), feature = "std"))]
pub type DefaultDevice = GetRandom;

/// This is a random device that generates seeds by reading from /dev/urandom
#[cfg(all(unix, feature = "std"))]
pub struct DevUrandom {
    dev_random: File,
}

#[cfg(all(unix, feature = "std"))]
impl DevUrandom {
    /// Creates a new `DevUrandom` device.
    ///
    /// # Panics
    ///
    /// Panics if the device is not found or cannot be read from.
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
impl RandomDevice for DevUrandom {
    fn seed_bytes<const N: usize>(&mut self) -> [u8; N] {
        let mut result = [0; N];
        self.dev_random
            .read_exact(&mut result)
            .expect("Failed to read from /dev/urandom");
        assert!(
            result.iter().any(|v| *v != 0),
            "Entropy source generated all zeros!"
        );
        result
    }
}

/// This is a random device that generates seeds using the getrandom crate.
#[cfg(all(not(unix), feature = "std"))]
pub struct GetRandom;

#[cfg(all(not(unix), feature = "std"))]
impl GetRandom {
    /// Creates a new `GetRandom` device
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(all(not(unix), feature = "std"))]
impl Default for GetRandom {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(not(unix), feature = "std"))]
impl RandomDevice for GetRandom {
    fn seed_bytes<const N: usize>(&mut self) -> [u8; N] {
        let mut result = [0; N];
        getrandom::fill(&mut result).expect("getrandom::fill failed");
        assert!(
            result.iter().any(|v| *v != 0),
            "getrandom generated all zeros!"
        );
        result
    }
}

/// This is a device that generates an arbitrary length output from a u64 seed
/// using the SplitMix algorithm from <https://prng.di.unimi.it/splitmix64.c>
pub struct SplitMixDevice {
    state: u64,
}

impl SplitMixDevice {
    /// Creates a new SplitMixDevice using a u64 seed.
    ///
    /// # Arguments
    ///
    /// * `seed`: The seed value to initialize with
    ///
    /// returns: SplitMixDevice
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

impl RandomDevice for SplitMixDevice {
    fn seed_bytes<const N: usize>(&mut self) -> [u8; N] {
        let mut out_inx: usize = 0;
        let mut output = [0; N];
        while out_inx < N {
            let num = usize::min(8, N - out_inx);
            // The endianness used here should match that used in FromRaw:
            output[out_inx..(out_inx + num)].copy_from_slice(&self.next().to_be_bytes()[0..num]);
            out_inx += num;
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RandomDevice;

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

    #[cfg(all(not(unix), feature = "std"))]
    #[test]
    fn generate_64_bit_seed_with_gev_random() {
        let seed1: u64 = GetRandom::new().seed();
        let seed2: u64 = GetRandom::new().seed();
        assert_ne!(seed1, seed2);
    }

    #[test]
    fn test_splitmix() {
        let mut dev = SplitMixDevice::new(42);
        let output: [u8; 12] = dev.seed_bytes();
        assert_eq!(
            output,
            [189, 215, 50, 38, 47, 235, 110, 149, 40, 239, 227, 51]
        );
    }
}
