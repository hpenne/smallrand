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
    repetition_count_tester: RepetitionCountTester,
    adaptive_proportion_tester: AdaptiveProportionTester,
}

impl<T> CheckedDevice<T>
where
    T: RandomDevice,
{
    fn new(mut wrapped_device: T) -> Self {
        let mut previous = [0; 8];
        wrapped_device.fill(&mut previous);
        let mut repetition_count_tester = RepetitionCountTester::default();
        repetition_count_tester.test(&previous);
        let mut adaptive_proportion_tester = AdaptiveProportionTester::default();
        adaptive_proportion_tester.test(&previous);
        Self {
            previous,
            device: wrapped_device,
            repetition_count_tester,
            adaptive_proportion_tester,
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

        self.device.fill(destination);

        // Check that the 8 bytes we fetched above are not present in the
        // output we want to return:
        assert!(
            !destination
                .windows(new_random.len())
                .any(|candidate| candidate == new_random),
            "The entropy source is broken (found earlier data as a substring in new data)"
        );

        // Run the NIST SP 800-90B "Repetition Count Test" (see section 4.4.1)
        self.repetition_count_tester.test(&new_random);
        self.repetition_count_tester.test(destination);

        // Run the NIST SP 800-90B "Adaptive Proportion Test" (see section 4.4.2)
        self.adaptive_proportion_tester.test(&new_random);
        self.adaptive_proportion_tester.test(destination);

        self.previous = new_random;
    }
}

// This is the Repetition Count Test algorithm from NIST 800-90B section 4.4.1
#[derive(Default)]
struct RepetitionCountTester {
    current_value: Option<u8>,
    num_found: usize,
}

impl RepetitionCountTester {
    // NIST SP 800-90B section 4.4 proposes that 1:2^20 is a reasonable
    // false positive probability.
    // If we assume that the source has full entropy, then this means that
    // an error requires four identical samples.
    const REPEAT_THRESHOLD: usize = 4;

    fn test(&mut self, data: &[u8]) {
        let mut i = data.iter();
        if self.current_value.is_none() {
            self.current_value = Some(*i.next().expect("This function requires a non-empty slice"));
            self.num_found = 1;
        }
        for x in i {
            if *x == self.current_value.unwrap() {
                self.num_found += 1;
                assert!(
                    self.num_found < Self::REPEAT_THRESHOLD,
                    "Repetition Count Test failed"
                );
            } else {
                self.current_value = Some(*x);
                self.num_found = 1;
            }
        }
    }
}

// This is the "Adaptive Proportion Test" algorithm from NIST 800-90B section 4.4.2
#[derive(Default)]
struct AdaptiveProportionTester {
    value_to_count: u8,
    num_found: usize,
    num_processed: usize,
}

impl AdaptiveProportionTester {
    // NIST SP 800-90B section 4.4 proposes that 1:2^20 is a reasonable
    // false positive probability, which results in these constants (section 4.4.2):
    const MAX_NUM: usize = 13;
    const WINDOW_SIZE: usize = 512;

    fn test(&mut self, data: &[u8]) {
        for i in data {
            match self.num_processed {
                0 => self.value_to_count = *i,
                Self::WINDOW_SIZE => {
                    // We're throwing away this value,
                    // to avoid aligning each block on a 512 byte boundary.
                    self.num_processed = 0;
                    self.num_found = 0;
                }
                _ => {
                    if self.value_to_count == *i {
                        self.num_found += 1;
                    }
                    self.num_processed += 1;
                    assert!(
                        self.num_found < Self::MAX_NUM,
                        "Adaptive Proportion Test failed"
                    );
                }
            }
        }
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
        data: Vec<Vec<u8>>,
    }

    impl TestDevice {
        fn new(data: Vec<Vec<u8>>) -> Self {
            Self { data }
        }
    }

    impl RandomDevice for TestDevice {
        fn fill(&mut self, destination: &mut [u8]) {
            destination.copy_from_slice(&self.data.first().unwrap());
            self.data.remove(0);
        }
    }

    #[test]
    fn none_repeating_device_is_accepted() {
        let mut output = [0_u8; 16];
        SecureDevice::new().fill(&mut output);
        let mut device = CheckedDevice::new(TestDevice::new(vec![
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![8, 9, 10, 11, 12, 13, 14, 15],
            vec![
                16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
            ],
        ]));
        device.fill(&mut output);
    }

    #[test]
    fn repeating_device_is_detected() {
        let mut output = [0_u8; 8];
        SecureDevice::new().fill(&mut output);
        let mut device = CheckedDevice::new(TestDevice::new(vec![
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![0, 1, 2, 3, 4, 5, 6, 7],
        ]));
        let result = std::panic::catch_unwind(move || {
            device.fill(&mut output);
        });
        assert!(result.is_err());
    }

    #[test]
    fn repeated_substring_is_detected() {
        let mut device = CheckedDevice::new(TestDevice::new(vec![
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![8, 9, 10, 11, 12, 13, 14, 15],
            vec![16, 17, 18, 19, 8, 9, 10, 11, 12, 13, 14, 15, 28, 29, 30, 31],
        ]));
        let mut output = [0_u8; 16];
        let result = std::panic::catch_unwind(move || {
            device.fill(&mut output);
        });
        assert!(result.is_err());
    }

    #[test]
    fn repetitions_are_detected1() {
        let mut device = CheckedDevice::new(TestDevice::new(vec![
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![8, 9, 10, 11, 12, 13, 14, 15],
            vec![
                16, 17, 18, 19, 20, 20, 20, 20, 24, 25, 26, 27, 28, 29, 30, 31,
            ],
        ]));
        let mut output = [0_u8; 16];
        let result = std::panic::catch_unwind(move || {
            device.fill(&mut output);
        });
        assert!(result.is_err());
    }

    #[test]
    fn repetitions_are_detected2() {
        let mut device = CheckedDevice::new(TestDevice::new(vec![
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![8, 9, 10, 11, 12, 13, 14, 15],
            vec![
                // The previous block ends with a 15, so that makes 4 in total
                15, 15, 15, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
            ],
        ]));
        let mut output = [0_u8; 16];
        let result = std::panic::catch_unwind(move || {
            device.fill(&mut output);
        });
        assert!(result.is_err());
    }

    #[test]
    fn repetitions_are_detected3() {
        let mut device = CheckedDevice::new(TestDevice::new(vec![
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![
                // The previous block ends with a 7, so that makes 4 in total
                7, 7, 7, 11, 12, 13, 14, 15,
            ],
        ]));
        let mut output = [0_u8; 16];
        let result = std::panic::catch_unwind(move || {
            device.fill(&mut output);
        });
        assert!(result.is_err());
    }
}
