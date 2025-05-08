#![cfg(feature = "std")]
use crate::{DefaultEntropy, EntropySource};
use std::sync::{Mutex, OnceLock};

/// This is an `EntropySource` (entropy source for seeds) which
/// uses a [DefaultEntropy] as its source of data, but performs security
/// tests on the data to check that the device is not broken.
///
/// These tests include the Health Test in Section 4.4 of NIST SP 800-90B.
///
/// Note that [SecureEntropy] is just a proxy for a global shared device,
/// so tests for repeats of earlier samples still work even if
/// a new [SecureEntropy] is created for each use.
#[derive(Default)]
pub struct SecureEntropy;

impl SecureEntropy {
    /// Creates a new `SecureEntropy`
    ///
    /// returns: A new `SecureEntropy`
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl EntropySource for SecureEntropy {
    fn fill(&mut self, destination: &mut [u8]) {
        SECURE_DEVICE_IMPL
            .get_or_init(|| Mutex::new(CheckedDevice::new(DefaultEntropy::new())))
            .lock()
            .unwrap()
            .fill(destination);
    }
}

static SECURE_DEVICE_IMPL: OnceLock<Mutex<CheckedDevice<DefaultEntropy>>> = OnceLock::new();

struct CheckedDevice<T>
where
    T: EntropySource,
{
    previous: [u8; 8],
    device: T,
    repetition_count_tester: RepetitionCountTester,
    adaptive_proportion_tester: AdaptiveProportionTester,
}

impl<T> CheckedDevice<T>
where
    T: EntropySource,
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

impl<T> EntropySource for CheckedDevice<T>
where
    T: EntropySource,
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
                0 => {
                    self.value_to_count = *i;
                    self.num_found = 1;
                    self.num_processed = 1;
                }
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
        SecureEntropy::new().fill(&mut output);
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

    impl EntropySource for TestDevice {
        fn fill(&mut self, destination: &mut [u8]) {
            destination.copy_from_slice(&self.data.first().unwrap());
            self.data.remove(0);
        }
    }

    #[test]
    fn none_repeating_device_is_accepted() {
        let mut output = [0_u8; 16];
        SecureEntropy::new().fill(&mut output);
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
        SecureEntropy::new().fill(&mut output);
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

    #[test]
    fn adaptive_proportion_test_accepts_12_repeats() {
        let vec1 = vec![0, 1, 2, 0, 4, 5, 6, 7]; // Two 0s here
        let vec2 = vec![8, 9, 0, 11, 12, 13, 14, 15]; // One 0 here
        let mut vec3: [u8; 512] = core::array::from_fn(|i| (i + 16) as u8);
        // Note: There is a zero already at 240.
        for inx in [53, 93, 123, 135, 147, 254, 275, 328] {
            vec3[inx] = 0;
        }
        let mut device = CheckedDevice::new(TestDevice::new(vec![vec1, vec2, vec3.into()]));
        let mut output = [0_u8; 512];
        device.fill(&mut output);
    }

    #[test]
    fn adaptive_proportion_test_detects_13_repeats_inside_512_window() {
        let vec1 = vec![0, 1, 2, 0, 4, 5, 6, 7]; // Two 0s here
        let vec2 = vec![8, 9, 0, 11, 12, 13, 14, 15]; // One 0 here
        let mut vec3: [u8; 512] = core::array::from_fn(|i| (i + 16) as u8);
        // Note: There is a zero already at 240. The zero at 495 is just inside the window
        for inx in [53, 93, 123, 135, 147, 254, 275, 328, 495] {
            vec3[inx] = 0;
        }
        let mut device = CheckedDevice::new(TestDevice::new(vec![vec1, vec2, vec3.into()]));
        let mut output = [0_u8; 512];
        let result = std::panic::catch_unwind(move || {
            device.fill(&mut output);
        });
        assert!(result.is_err());
    }

    #[test]
    fn adaptive_proportion_test_window_ends_after_512() {
        let vec1 = vec![0, 1, 2, 0, 4, 5, 6, 7]; // Two 0s here
        let vec2 = vec![8, 9, 0, 11, 12, 13, 14, 15]; // One 0 here
        let mut vec3: [u8; 512] = core::array::from_fn(|i| (i + 16) as u8);
        // Note: There is a zero already at 240. Last 0 is outside the window:
        for inx in [53, 93, 123, 135, 147, 254, 275, 328, 496] {
            vec3[inx] = 0;
        }
        let mut device = CheckedDevice::new(TestDevice::new(vec![vec1, vec2, vec3.into()]));
        let mut output = [0_u8; 512];
        device.fill(&mut output);
    }

    #[test]
    fn adaptive_proportion_test_second_window_starts_at_513() {
        let vec1 = vec![0, 1, 2, 0, 4, 5, 6, 7]; // 0 is the counted value. Two 0s here
        let vec2 = vec![8, 9, 0, 11, 12, 13, 14, 15]; // One 0 here
        let vec3: [u8; 512 - 16] = core::array::from_fn(|i| (i + 16) as u8);
        let vec4 = vec![16, 42, 18, 19, 20, 21, 22, 23]; // 16 is skipped, 42 is the counted value in window #2.
        let mut vec5: [u8; 512] = core::array::from_fn(|i| i as u8);

        // Note: There is a 42 already at 42 and 42 + 256 = 298. Last 42 is just inside window:
        for inx in [53, 93, 123, 135, 147, 254, 275, 328, 420, 512 - 9] {
            vec5[inx] = 42;
        }
        let mut device = CheckedDevice::new(TestDevice::new(vec![
            vec1,
            vec2,
            vec3.into(),
            vec4,
            vec5.into(),
        ]));
        let mut output1 = [0_u8; 496];
        device.fill(&mut output1);
        let mut output2 = [0_u8; 512];
        let result = std::panic::catch_unwind(move || {
            device.fill(&mut output2);
        });
        assert!(result.is_err());
    }
}
