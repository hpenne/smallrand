use std::fs::File;
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

pub struct DevRandom {
    dev_random: File,
}

impl DevRandom {
    fn new() -> Self {
        Self {
            dev_random: File::open("/dev/random").expect("Failed to open /dev/random"),
        }
    }
}

impl RandomDevice for DevRandom {
    fn seed_bytes<const N: usize>(&mut self) -> [u8; N] {
        let mut result = [0; N];
        self.dev_random
            .read_exact(&mut result)
            .expect("Failed to read from /dev/random");
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::devices::{DevRandom, RandomDevice};

    #[cfg(unix)]
    #[test]
    fn generate_64_bit_seed() {
        let seed1: u64 = DevRandom::new().seed();
        let seed2: u64 = DevRandom::new().seed();
        assert_ne!(seed1, seed2);
    }

    #[test]
    fn generate_128_bit_seed() {
        let seed1: u64 = DevRandom::new().seed();
        let seed2: u64 = DevRandom::new().seed();
        assert_ne!(seed1, seed2);
    }
}
