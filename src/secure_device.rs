#![cfg(feature = "std")]
use crate::{DefaultDevice, RandomDevice};
use std::sync::{Mutex, OnceLock};

pub struct SecureDevice;

impl RandomDevice for SecureDevice {
    fn seed_bytes<const N: usize>(&mut self) -> [u8; N] {
        SECURE_DEVICE_IMPL
            .get_or_init(|| Mutex::new(CheckedDevice::new(DefaultDevice::new())))
            .lock()
            .unwrap()
            .seed_bytes()
    }
}

static SECURE_DEVICE_IMPL: OnceLock<Mutex<CheckedDevice<DefaultDevice>>> = OnceLock::new();

pub struct CheckedDevice<T>
where
    T: RandomDevice,
{
    previous: [u8; 8],
    device: T,
}

impl<T> CheckedDevice<T>
where
    T: RandomDevice,
{
    fn new(mut wrapped_device: T) -> Self {
        Self {
            previous: wrapped_device.seed_bytes(),
            device: wrapped_device,
        }
    }
}

impl<T> RandomDevice for CheckedDevice<T>
where
    T: RandomDevice,
{
    fn seed_bytes<const N: usize>(&mut self) -> [u8; N] {
        // Ensure that the entropy source does not repeat itself:
        let new_random = self.device.seed_bytes();
        assert_ne!(new_random, self.previous);
        self.previous = new_random;

        self.device.seed_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct RepeatingDevice;

    impl RandomDevice for RepeatingDevice {
        fn seed_bytes<const N: usize>(&mut self) -> [u8; N] {
            core::array::from_fn(|i| i as u8)
        }
    }

    #[test]
    fn repeating_device_is_detected() {
        let mut device = CheckedDevice::new(RepeatingDevice::default());
        let result = std::panic::catch_unwind(move || {
            let _entropy: [u8; 5] = device.seed_bytes();
        });
        assert!(result.is_err());
    }
}
