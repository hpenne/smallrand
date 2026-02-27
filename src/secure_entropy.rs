//! `SecureEntropy` is an entropy source implementation that extends `EntropySource`.
//! It leverages a [`DefaultEntropy`] source while performing additional security
//! validations to ensure the integrity of the entropy data. These validations
//! help identify potential failures in the entropy source.
//!
//! The implementation follows guidelines from the [NIST SP 800-90B] standard,
//! specifically focusing on the Repetition Count Test and Adaptive Proportion Test.
//!
//! ### Features:
//! * Performs security checks on entropy data using NIST SP 800-90B-compliant tests.
//! * Ensures that output data does not repeat or contain previously generated sequences.
//! * Proxy design ensures that security checks remain effective even when multiple
//!   `SecureEntropy` instances are created.
//!
//! This struct provides appropriate default implementations via `new` and `default`
//! while also enabling custom panic handling via failure handlers.
#![cfg(feature = "std")]

use crate::{DefaultEntropy, EntropySource};
use std::convert::Infallible;
use std::mem;
use std::sync::{Mutex, OnceLock};

/// This is an `EntropySource` (entropy source for seeds) which
/// uses a [DefaultEntropy] as its source of data, but performs security
/// tests on the data to check that the entropy source is not broken.
///
/// These tests include the Health Test in Section 4.4 of NIST SP 800-90B.
///
/// Beware that the Repetition Count Test has a false positive rate of 1:2^24.
/// This may sound very unlikely, but if you construct a lot of random generators
/// using this source in CI testing, then failures will happen eventually.
/// This is particularly likely if you write a fuzzer where each iteration
/// creates a new random generator using this source.
///
/// Note that [SecureEntropy] is just a proxy for a global shared entropy source,
/// so tests for repeats of earlier samples still work even if
/// a new [SecureEntropy] is created for each use.
pub struct SecureEntropy {
    #[allow(clippy::type_complexity)]
    failure_handler: Option<Box<dyn FnOnce(&str) -> Infallible>>,
}

impl SecureEntropy {
    /// Creates a new `SecureEntropy`.
    ///
    /// Note that if the source fails statistics tests during calls to `fill` then
    /// it will panic.
    /// Such a panic will poison the internal mutex protecting the global shared source,
    /// causing future calls to `fill` (on any instance of SecureEntropy) to panic.
    /// Consider using `new_with_failure_handler` if you need to handle failures differently.
    ///
    /// returns: A new `SecureEntropy`
    ///
    /// # Panics
    #[must_use]
    pub fn new() -> Self {
        Self {
            failure_handler: Some(Box::new(|msg| panic!("{}", msg))),
        }
    }

    /// Creates a new `SecureEntropy` with a failure handler.
    ///
    /// # Arguments
    ///
    /// * `handler`: The failure handler.
    ///   This function will be called (instead of panicking) if the entropy source
    ///   fails security tests during calls to `fill`.
    ///   The handler must log/report the error and terminate the program,
    ///   since the global entropy source is broken and should not be used further.
    ///
    /// returns: A new `SecureEntropy`
    #[must_use]
    pub fn new_with_failure_handler<T>(handler: T) -> Self
    where
        T: FnOnce(&str) -> Infallible + Sized + 'static,
    {
        Self {
            failure_handler: Some(Box::new(handler)),
        }
    }
}

impl EntropySource for SecureEntropy {
    #[allow(unreachable_code)]
    fn fill(&mut self, destination: &mut [u8]) {
        if let Err(msg) = SECURE_ENTROPY_IMPL
            .get_or_init(|| {
                Mutex::new(
                    EntropyChecker::new(DefaultEntropy::new()).unwrap_or_else(|msg| {
                        if let Some(handler) = mem::take(&mut self.failure_handler) {
                            handler(msg);
                        }
                        unreachable!("The handler will terminate before we get here.");
                    }),
                )
            })
            .lock()
            .unwrap()
            .try_fill(destination)
        {
            if let Some(handler) = mem::take(&mut self.failure_handler) {
                handler(msg);
            } else {
                // This should never happen, as the handler should terminate
                panic!("failure_handler already consumed (source has already failed previously)!");
            }
        }
    }
}

impl Default for SecureEntropy {
    fn default() -> Self {
        Self::new()
    }
}

static SECURE_ENTROPY_IMPL: OnceLock<Mutex<EntropyChecker<DefaultEntropy>>> = OnceLock::new();

struct EntropyChecker<T>
where
    T: EntropySource,
{
    previous: [u8; 8],
    entropy_source: T,
    repetition_count_tester: RepetitionCountTester,
    adaptive_proportion_tester: AdaptiveProportionTester,
}

impl<T> EntropyChecker<T>
where
    T: EntropySource,
{
    fn new(mut wrapped_source: T) -> Result<Self, &'static str> {
        let mut previous = [0; 8];
        wrapped_source.fill(&mut previous);
        let mut repetition_count_tester = RepetitionCountTester::default();
        repetition_count_tester.test(&previous)?;
        let mut adaptive_proportion_tester = AdaptiveProportionTester::default();
        adaptive_proportion_tester.test(&previous)?;
        Ok(Self {
            previous,
            entropy_source: wrapped_source,
            repetition_count_tester,
            adaptive_proportion_tester,
        })
    }

    fn try_fill(&mut self, destination: &mut [u8]) -> Result<(), &'static str> {
        // Ensure that the entropy source does not repeat itself,
        // by getting 8 bytes every time and comparing them to
        // the 8 bytes from last time.
        // Notice that we pull out dedicated random data for this,
        // so the data we hold on to is not part of the data that we
        // return to the user. For security reasons, we do not want this code
        // to retain random data that could be used for encryption keys or other
        // security-critical uses by the client code.
        let mut new_random = [0; 8];
        self.entropy_source.fill(&mut new_random);
        if new_random == self.previous {
            return Err(
                "SecureEntropy: The entropy source is broken (repeats 8 byte data sequence)",
            );
        }

        // Run the NIST SP 800-90B "Repetition Count Test" (see section 4.4.1)
        // on the 8 samples we fetched above:
        self.repetition_count_tester.test(&new_random)?;

        // Run the NIST SP 800-90B "Adaptive Proportion Test" (see section 4.4.2)
        // on the 8 samples we fetched above:
        self.adaptive_proportion_tester.test(&new_random)?;

        // Get the samples to return
        self.entropy_source.fill(destination);

        // Check that the 8 samples we fetched first are not present in the
        // output we want to return:
        if destination
            .windows(new_random.len())
            .any(|candidate| candidate == new_random)
        {
            return Err("SecureEntropy: The entropy source is broken (found earlier data as a substring in new data)");
        }

        // Run the NIST SP 800-90B "Repetition Count Test" (see section 4.4.1)
        self.repetition_count_tester.test(destination)?;

        // Run the NIST SP 800-90B "Adaptive Proportion Test" (see section 4.4.2)
        self.adaptive_proportion_tester.test(destination)?;

        self.previous = new_random;

        Ok(())
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
    // If we assume that the source has full entropy (1/256 per value), then:
    //   P(run of length k) = 1 / 2^(8*(k-1))
    // We need to subtract 1 from k because the first sample is always equal to itself.
    //   k=3 -> 1:2^16 (too likely), k=4 -> 1:2^24 (below 1:2^20)
    // So the test must fail when a run of 4 identical samples is observed.
    const REPEAT_THRESHOLD: usize = 4;

    fn test(&mut self, data: &[u8]) -> Result<(), &'static str> {
        let mut i = data.iter();
        if self.current_value.is_none() {
            self.current_value = match i.next() {
                None => return Ok(()),
                Some(&value) => Some(value),
            };
            self.num_found = 1;
        }
        for x in i {
            if Some(*x) == self.current_value {
                self.num_found += 1;
                if self.num_found >= Self::REPEAT_THRESHOLD {
                    return Err("SecureEntropy: Repetition Count Test failed");
                }
            } else {
                self.current_value = Some(*x);
                self.num_found = 1;
            }
        }
        Ok(())
    }
}

// This is the "Adaptive Proportion Test" algorithm from NIST 800-90B section 4.4.2
#[derive(Default)]
struct AdaptiveProportionTester {
    state: AdaptiveProportionTesterState,
}

#[derive(Default)]
enum AdaptiveProportionTesterState {
    #[default]
    InitializeWindow,
    ProcessWindow {
        value_to_count: u8,
        num_found: usize,
        num_processed: usize,
    },
    SkipNext,
}

impl AdaptiveProportionTester {
    // NIST SP 800-90B section 4.4 proposes that 1:2^20 is a reasonable
    // false positive probability, which results in these constants (section 4.4.2):
    const MAX_NUM: usize = 13;
    const WINDOW_SIZE: usize = 512;

    fn test(&mut self, data: &[u8]) -> Result<(), &'static str> {
        for sample in data {
            self.state = match self.state {
                AdaptiveProportionTesterState::InitializeWindow => Self::new_window(*sample),
                AdaptiveProportionTesterState::ProcessWindow {
                    value_to_count,
                    mut num_found,
                    mut num_processed,
                } => {
                    if value_to_count == *sample {
                        num_found += 1;
                    }
                    num_processed += 1;
                    if num_found >= Self::MAX_NUM {
                        return Err("SecureEntropy: Adaptive Proportion Test failed");
                    }
                    if num_processed == Self::WINDOW_SIZE {
                        AdaptiveProportionTesterState::SkipNext
                    } else {
                        AdaptiveProportionTesterState::ProcessWindow {
                            value_to_count,
                            num_found,
                            num_processed,
                        }
                    }
                }
                AdaptiveProportionTesterState::SkipNext => {
                    // Skip this value to avoid aligning all windows on 512 bytes,
                    // and set up to start a new value on the next sample:
                    AdaptiveProportionTesterState::InitializeWindow {}
                }
            };
        }
        Ok(())
    }

    fn new_window(sample: u8) -> AdaptiveProportionTesterState {
        AdaptiveProportionTesterState::ProcessWindow {
            value_to_count: sample,
            num_found: 1,
            num_processed: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[test]
    fn secure_source_generates_non_zero_data() {
        let mut output = [0_u8; 8];
        SecureEntropy::new().fill(&mut output);
        assert_ne!([0_u8; 8], output);
    }

    #[derive(Default)]
    struct TestSource {
        data: VecDeque<Vec<u8>>,
    }

    impl TestSource {
        fn new(data: Vec<Vec<u8>>) -> Self {
            Self {
                data: VecDeque::from(data),
            }
        }
    }

    impl EntropySource for TestSource {
        fn fill(&mut self, destination: &mut [u8]) {
            destination.copy_from_slice(&self.data.pop_front().unwrap());
        }
    }

    #[test]
    fn none_repeating_source_is_accepted() {
        let mut output = [0_u8; 16];
        let mut entropy_source = EntropyChecker::new(TestSource::new(vec![
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![8, 9, 10, 11, 12, 13, 14, 15],
            vec![
                16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
            ],
        ]))
        .unwrap();
        assert!(entropy_source.try_fill(&mut output).is_ok());
    }

    #[test]
    fn repeating_source_is_detected() {
        let mut entropy_source = EntropyChecker::new(TestSource::new(vec![
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![0, 1, 2, 3, 4, 5, 6, 7],
        ]))
        .unwrap();
        let mut output = [0_u8; 16];
        assert!(entropy_source.try_fill(&mut output).is_err());
    }

    #[test]
    fn repeated_substring_is_detected() {
        let mut entropy_source = EntropyChecker::new(TestSource::new(vec![
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![8, 9, 10, 11, 12, 13, 14, 15],
            vec![16, 17, 18, 19, 8, 9, 10, 11, 12, 13, 14, 15, 28, 29, 30, 31],
        ]))
        .unwrap();
        let mut output = [0_u8; 16];
        assert!(entropy_source.try_fill(&mut output).is_err());
    }

    #[test]
    fn three_repetitions_are_accepted() {
        let mut entropy_source = EntropyChecker::new(TestSource::new(vec![
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![8, 9, 10, 11, 12, 13, 14, 15],
            vec![
                16, 17, 18, 19, 20, 20, 20, 23, 24, 25, 26, 27, 28, 29, 30, 31,
            ],
        ]))
        .unwrap();
        let mut output = [0_u8; 16];
        assert!(entropy_source.try_fill(&mut output).is_ok());
    }

    #[test]
    fn four_repetitions_are_detected1() {
        let mut entropy_source = EntropyChecker::new(TestSource::new(vec![
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![8, 9, 10, 11, 12, 13, 14, 15],
            vec![
                16, 17, 18, 19, 20, 20, 20, 20, 24, 25, 26, 27, 28, 29, 30, 31,
            ],
        ]))
        .unwrap();
        let mut output = [0_u8; 16];
        assert!(entropy_source.try_fill(&mut output).is_err());
    }

    #[test]
    fn four_repetitions_are_detected2() {
        let mut entropy_source = EntropyChecker::new(TestSource::new(vec![
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![8, 9, 10, 11, 12, 13, 14, 15],
            vec![
                // The previous block ends with a 15, so that makes 4 in total
                15, 15, 15, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
            ],
        ]))
        .unwrap();
        let mut output = [0_u8; 16];
        assert!(entropy_source.try_fill(&mut output).is_err());
    }

    #[test]
    fn four_repetitions_are_detected3() {
        let mut entropy_source = EntropyChecker::new(TestSource::new(vec![
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![
                // The previous block ends with a 7, so that makes 5 in total
                7, 7, 7, 11, 12, 13, 14, 15,
            ],
        ]))
        .unwrap();
        let mut output = [0_u8; 16];
        assert!(entropy_source.try_fill(&mut output).is_err());
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
        let mut entropy_source =
            EntropyChecker::new(TestSource::new(vec![vec1, vec2, vec3.into()])).unwrap();
        let mut output = [0_u8; 512];
        assert!(entropy_source.try_fill(&mut output).is_ok());
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
        let mut entropy_source =
            EntropyChecker::new(TestSource::new(vec![vec1, vec2, vec3.into()])).unwrap();
        let mut output = [0_u8; 512];
        assert!(entropy_source.try_fill(&mut output).is_err());
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
        let mut entropy_source =
            EntropyChecker::new(TestSource::new(vec![vec1, vec2, vec3.into()])).unwrap();
        let mut output = [0_u8; 512];
        assert!(entropy_source.try_fill(&mut output).is_ok());
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
        let mut entropy_source = EntropyChecker::new(TestSource::new(vec![
            vec1,
            vec2,
            vec3.into(),
            vec4,
            vec5.into(),
        ]))
        .unwrap();
        let mut output1 = [0_u8; 496];
        assert!(entropy_source.try_fill(&mut output1).is_ok());
        let mut output2 = [0_u8; 512];
        assert!(entropy_source.try_fill(&mut output2).is_err());
    }
}
