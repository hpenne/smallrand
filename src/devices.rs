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

#[cfg(test)]
mod tests {
    use super::*;

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
}
