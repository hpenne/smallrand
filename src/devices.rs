#[cfg(all(unix, feature = "std"))]
use std::fs::File;
#[cfg(all(unix, feature = "std"))]
use std::io::Read;

pub trait RandomDevice {
    fn seed_bytes<const N: usize>(&mut self) -> [u8; N];

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

#[cfg(all(unix, feature = "std"))]
pub struct DevRandom {
    dev_random: File,
}

#[cfg(all(unix, feature = "std"))]
impl DevRandom {
    pub fn new() -> Self {
        Self {
            dev_random: File::open("/dev/random").expect("Failed to open /dev/random"),
        }
    }
}

#[cfg(all(unix, feature = "std"))]
impl Default for DevRandom {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(unix, feature = "std"))]
impl RandomDevice for DevRandom {
    fn seed_bytes<const N: usize>(&mut self) -> [u8; N] {
        let mut result = [0; N];
        self.dev_random
            .read_exact(&mut result)
            .expect("Failed to read from /dev/random");
        result
    }
}

#[cfg(feature = "getrandom")]
pub struct GetRandom;

#[cfg(feature = "getrandom")]
impl GetRandom {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(feature = "getrandom")]
impl Default for GetRandom {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "getrandom")]
impl RandomDevice for GetRandom {
    fn seed_bytes<const N: usize>(&mut self) -> [u8; N] {
        let mut result = [0; N];
        getrandom::fill(&mut result).expect("getrandom::fill failed");
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(all(unix, feature = "std"))]
    #[test]
    fn generate_64_bit_seed_with_dev_random() {
        let seed1: u64 = DevRandom::new().seed();
        let seed2: u64 = DevRandom::new().seed();
        assert_ne!(seed1, seed2);
    }

    #[cfg(all(unix, feature = "std"))]
    #[test]
    fn generate_128_bit_seed_dev_random() {
        let seed1: u64 = DevRandom::new().seed();
        let seed2: u64 = DevRandom::new().seed();
        assert_ne!(seed1, seed2);
    }

    #[cfg(feature = "getrandom")]
    #[test]
    fn generate_64_bit_seed_with_gev_random() {
        let seed1: u64 = GetRandom::new().seed();
        let seed2: u64 = GetRandom::new().seed();
        assert_ne!(seed1, seed2);
    }
}
