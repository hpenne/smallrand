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
    fn fill(&mut self, destination: &mut [u8]) {
        SECURE_DEVICE_IMPL
            .get_or_init(|| Mutex::new(CheckedDevice::new(DefaultDevice::new())))
            .lock()
            .unwrap()
            .fill(destination);
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
        let mut previous = [0; 8];
        wrapped_device.fill(&mut previous);
        Self {
            previous,
            device: wrapped_device,
        }
    }
}

impl<T> RandomDevice for CheckedDevice<T>
where
    T: RandomDevice,
{
    fn fill(&mut self, destination: &mut [u8]) {
        // Ensure that the entropy source does not repeat itself,
        // by getting 8 bytes every time and comparing them to
        // the 8 bytes from last time.
        // Notice that we pull out dedicated random data for this,
        // so the data we hold on to is not part of the data that we
        // return to the user. For security reasons, we do not want this code
        // to retain random data that could be used for encryption keys or other
        // security critical uses by the client code.
        let mut new_random = [0; 8];
        self.device.fill(&mut new_random);
        assert_ne!(
            new_random, self.previous,
            "The entropy source is broken (repeats data)"
        );
        self.previous = new_random;

        self.device.fill(destination);

        // Check that the 8 bytes we fetched above are not present in the
        // output we want to return:
        assert!(
            !destination
                .windows(self.previous.len())
                .any(|candidate| candidate == self.previous),
            "The entropy source is broken (found earlier data as a substring in new data)"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secure_device_generates_non_zero_data() {
        let mut output = [0_u8; 8];
        SecureDevice::new().fill(&mut output);
        assert_ne!([0_u8; 8], output);
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
        fn fill(&mut self, destination: &mut [u8]) {
            destination.copy_from_slice(&self.data.first().unwrap()[0..destination.len()]);
            self.data.remove(0);
        }
    }

    #[test]
    fn none_repeating_device_is_accepted() {
        let mut output = [0_u8; 8];
        SecureDevice::new().fill(&mut output);
        let mut device = CheckedDevice::new(TestDevice::new(vec![
            core::array::from_fn(|i| i as u8),
            core::array::from_fn(|i| (i + 16) as u8),
            core::array::from_fn(|i| (i + 32) as u8),
        ]));
        device.fill(&mut output);
    }

    #[test]
    fn repeating_device_is_detected() {
        let mut output = [0_u8; 8];
        SecureDevice::new().fill(&mut output);
        let mut device = CheckedDevice::new(TestDevice::new(vec![
            core::array::from_fn(|i| i as u8),
            core::array::from_fn(|i| i as u8),
            core::array::from_fn(|i| i as u8),
        ]));
        let result = std::panic::catch_unwind(move || {
            device.fill(&mut output);
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
        let mut output = [0_u8; 16];
        let result = std::panic::catch_unwind(move || {
            device.fill(&mut output);
        });
        assert!(result.is_err());
    }
}
