#![cfg(feature = "std")]
use crate::{DefaultDevice, RandomDevice};
use std::sync::{Mutex, OnceLock};

/// This is a RandomDevice (entropy source for seeds) which
/// uses a `DefaultDevice` as its source of data, but performs security
/// tests on the data to check that the device is not broken.
///
/// Note that `SecureDevice` is just a proxy for a global shared device,
/// so the tests for repeats of earlier samples still work even if
/// a new `SecureDevice` is created for each use.
#[derive(Default)]
pub struct SecureDevice;

impl SecureDevice {
    /// Creates a new `SecureDevice`
    ///
    /// returns: A new `SecureDevice`
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

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

struct CheckedDevice<T>
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
        // Ensure that the entropy source does not repeat itself,
        // by getting 8 bytes every time and comparing them to
        // the 8 bytes from last time.
        // Notice that we pull out dedicated random data for this,
        // so the data we hold on to is not part of the data that we
        // return to the user. For security reasons, we do not want this code
        // to retain random data that could be used for encryption keys or other
        // security critical uses by the client code.
        let new_random = self.device.seed_bytes();
        assert_ne!(
            new_random, self.previous,
            "The entropy source is broken (repeats data)"
        );
        self.previous = new_random;

        let output = self.device.seed_bytes();

        // Check that the 8 bytes we fetched above are not present in the
        // output we want to return:
        assert!(
            !output
                .windows(self.previous.len())
                .any(|candidate| candidate == self.previous),
            "The entropy source is broken (found earlier data as a substring in new data)"
        );

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secure_device_generates_non_zero_data() {
        assert_ne!([0_u8; 8], SecureDevice::new().seed_bytes())
    }

    #[derive(Default)]
    struct TestDevice {
        data: Vec<[u8; 16]>,
    }

    impl TestDevice {
        fn new(data: Vec<[u8; 16]>) -> Self {
            Self { data }
        }
    }

    impl RandomDevice for TestDevice {
        fn seed_bytes<const N: usize>(&mut self) -> [u8; N] {
            let mut output = [0; N];
            output.copy_from_slice(&self.data.first().unwrap()[0..N]);
            self.data.remove(0);
            output
        }
    }

    #[test]
    fn none_repeating_device_is_accepted() {
        let mut device = CheckedDevice::new(TestDevice::new(vec![
            core::array::from_fn(|i| i as u8),
            core::array::from_fn(|i| (i + 16) as u8),
            core::array::from_fn(|i| (i + 32) as u8),
        ]));
        let _entropy: [u8; 16] = device.seed_bytes();
    }

    #[test]
    fn repeating_device_is_detected() {
        let mut device = CheckedDevice::new(TestDevice::new(vec![
            core::array::from_fn(|i| i as u8),
            core::array::from_fn(|i| i as u8),
            core::array::from_fn(|i| i as u8),
        ]));
        let result = std::panic::catch_unwind(move || {
            let _entropy: [u8; 5] = device.seed_bytes();
        });
        assert!(result.is_err());
    }

    #[test]
    fn repeated_substring_is_detected() {
        let mut device = CheckedDevice::new(TestDevice::new(vec![
            core::array::from_fn(|i| i as u8),
            core::array::from_fn(|i| (i + 16) as u8),
            [
                32, 33, 34, 35, 36, 16, 17, 18, 19, 20, 21, 22, 23, 45, 46, 47,
            ],
        ]));
        let result = std::panic::catch_unwind(move || {
            let _entropy: [u8; 16] = device.seed_bytes();
        });
        assert!(result.is_err());
    }
}
