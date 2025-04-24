#![forbid(unsafe_code)]

use crate::ranges::GenerateRange;

/// This is the trait that all PRNGs must implement.
/// It declares two functions that PRNGs must implement (to generate u32 and u64 random values),
/// and based on these provides implementations of all the other
/// functions supported by the crate.
pub trait Rng {
    /// Generates a random u32.
    /// Used by other functions as input.
    fn random_u32(&mut self) -> u32;

    /// Generates a random u32.
    /// Used by other functions as input.
    fn random_u64(&mut self) -> u64;

    /// Generates a single random integer
    ///
    /// # Arguments
    ///
    /// returns: A random integer
    ///
    #[inline]
    fn random<T>(&mut self) -> T
    where
        T: ValueFromRng,
        Self: Sized,
    {
        T::value_from_rng(self)
    }

    /// Generates a single random integer or float in a specified range.
    /// The distribution is strictly uniform.
    /// The following types are supported:
    /// u8, u16, u64, u128, usize, i8, i16, i64, i128, isize, f32, f64
    ///
    /// Any kind of range is supported for integers, but only `Range` for floats.
    ///
    /// # Arguments
    ///
    /// * `range`: The range of the uniform distribution.
    ///
    /// returns: A random value in the range
    ///
    fn range<T>(&mut self, range: impl Into<GenerateRange<T>>) -> T
    where
        T: RangeFromRng,
        Self: Sized,
    {
        T::range_from_rng(self, range)
    }

    /// Provides an iterator that emits random values.
    ///
    /// returns: An iterator that outputs random values. Never None.
    ///
    #[inline]
    fn iter<T>(&mut self) -> impl Iterator<Item = T>
    where
        T: ValueFromRng,
        Self: Sized,
    {
        core::iter::from_fn(|| Some(self.random()))
    }

    /// Provides an iterator that emits random u8 values.
    /// Same as the generic variant, but more efficient.
    /// If you want to fill a slice of u8 with values,
    /// then the `fill_u8` function is even more performant.
    ///
    /// returns: An iterator that outputs random u8 values. Never None.
    ///
    #[inline]
    fn iter_u8(&mut self) -> impl Iterator<Item = u8>
    where
        Self: Sized,
    {
        self.iter::<u64>().flat_map(u64::to_ne_bytes)
    }

    /// Fills a mutable slice with random values.
    ///
    /// # Arguments
    ///
    /// * `destination`: The slice to fill
    ///
    #[inline]
    fn fill<T>(&mut self, destination: &mut [T])
    where
        T: ValueFromRng,
        Self: Sized,
    {
        for element in destination {
            *element = self.random();
        }
    }

    /// Fills a mutable slice of u8 with random values.
    /// Faster than `fill` for u8 values.
    ///
    /// # Arguments
    ///
    /// * `destination`: The slice to fill
    ///
    #[inline]
    fn fill_u8(&mut self, destination: &mut [u8])
    where
        Self: Sized,
    {
        let mut blocks = destination.chunks_exact_mut(core::mem::size_of::<u64>());
        for block in blocks.by_ref() {
            block.copy_from_slice(&self.random_u64().to_ne_bytes());
        }
        let bytes_remaining = blocks.into_remainder();
        if !bytes_remaining.is_empty() {
            bytes_remaining
                .copy_from_slice(&self.random::<u64>().to_ne_bytes()[..bytes_remaining.len()]);
        }
    }

    /// Shuffles the elements of a slice
    ///
    /// # Arguments
    ///
    /// * `target`: The slice to shuffle
    ///
    #[inline]
    fn shuffle<T>(&mut self, target: &mut [T])
    where
        T: Clone,
        Self: Sized,
    {
        // This is the forward version of the Fisher-Yates/Knuth shuffle:
        // https://en.wikipedia.org/wiki/Fisher–Yates_shuffle
        if !target.is_empty() {
            for inx in 0_usize..target.len() - 1 {
                // Note: "inx" is part of the range, to allow the current element to be swapped
                // with itself. Otherwise, it will always be moved, which would be incorrect.
                target.swap(inx, self.range(inx..target.len()));
            }
        }
    }
}

pub trait ValueFromRng {
    fn value_from_rng<T: Rng>(device: &mut T) -> Self;
}

impl ValueFromRng for bool {
    fn value_from_rng<T: Rng>(rng: &mut T) -> Self {
        rng.random_u32() & 1 == 1
    }
}

impl ValueFromRng for u8 {
    #[allow(clippy::cast_possible_truncation)]
    fn value_from_rng<T: Rng>(rng: &mut T) -> Self {
        rng.random_u32() as Self
    }
}

impl ValueFromRng for u16 {
    #[allow(clippy::cast_possible_truncation)]
    fn value_from_rng<T: Rng>(rng: &mut T) -> Self {
        rng.random_u32() as Self
    }
}

impl ValueFromRng for u32 {
    #[allow(clippy::cast_possible_truncation)]
    fn value_from_rng<T: Rng>(rng: &mut T) -> Self {
        rng.random_u32()
    }
}

impl ValueFromRng for u64 {
    fn value_from_rng<T: Rng>(rng: &mut T) -> Self {
        rng.random_u64()
    }
}

impl ValueFromRng for u128 {
    fn value_from_rng<T: Rng>(rng: &mut T) -> Self {
        (u128::from(rng.random_u64()) << 64) | u128::from(rng.random_u64())
    }
}

impl ValueFromRng for usize {
    #[cfg(target_pointer_width = "16")]
    #[allow(clippy::cast_possible_truncation)]
    fn value_from_rng<T: Rng>(rng: &mut T) -> Self {
        rng.random_u32() as usize
    }

    #[cfg(target_pointer_width = "32")]
    #[allow(clippy::cast_possible_truncation)]
    fn value_from_rng<T: Rng>(rng: &mut T) -> Self {
        rng.random_u32() as usize
    }

    #[cfg(target_pointer_width = "64")]
    #[allow(clippy::cast_possible_truncation)]
    fn value_from_rng<T: Rng>(rng: &mut T) -> Self {
        rng.random_u64() as usize
    }
}

pub trait RangeFromRng {
    fn range_from_rng<T: Rng>(device: &mut T, range: impl Into<GenerateRange<Self>>) -> Self
    where
        Self: Sized;
}

trait ZeroBasedRange {
    fn zero_based_range_from_rng(rng: &mut impl Rng, span: Self) -> Self;
}

macro_rules! zero_based_range_from_rng_lemire {
    ($output_type: ty, $bigger_type: ty) => {
        impl ZeroBasedRange for $output_type {
            #[inline]
            #[allow(clippy::cast_possible_truncation)]
            fn zero_based_range_from_rng(rng: &mut impl Rng, span: Self) -> Self {
                // Lemire's algorithm (https://lemire.me/blog/2016/06/30/fast-random-shuffling/)
                const SIZE_IN_BITS: usize = core::mem::size_of::<$output_type>() * 8;
                let m =
                    <$bigger_type>::from(rng.random::<$output_type>()) * <$bigger_type>::from(span);
                let mut high = (m >> SIZE_IN_BITS) as $output_type;
                let mut low = m as $output_type;
                if low < span {
                    let threshold = span.wrapping_neg() % span;
                    while low < threshold {
                        let m = <$bigger_type>::from(rng.random::<$output_type>())
                            * <$bigger_type>::from(span);
                        high = (m >> SIZE_IN_BITS) as $output_type;
                        low = m as $output_type;
                    }
                }
                high
            }
        }
    };
}

zero_based_range_from_rng_lemire!(u16, u32);
zero_based_range_from_rng_lemire!(u32, u64);
zero_based_range_from_rng_lemire!(u64, u128);

macro_rules! zero_based_range_from_rng {
    ($output_type: ty) => {
        impl ZeroBasedRange for u128 {
            #[inline]
            fn zero_based_range_from_rng(rng: &mut impl Rng, span: Self) -> Self {
                let mut random_value: Self = rng.random();
                let reduced_max = Self::MAX - span + 1;
                let max_valid_value = Self::MAX - (reduced_max % span);
                while random_value > max_valid_value {
                    random_value = rng.random();
                }
                random_value % span
            }
        }
    };
}

// We're using the simpler rejection sampling for u128.
// Lemire gets very complicated for u128 when there is no "u256"
// and the implementation would be virtually untestable.
// Also, the total speed difference on the "bigger" CPUs that are likely to
// need random u128s in a range is not that big (measured to 7% on an M1 for u64)
zero_based_range_from_rng!(u128);

macro_rules! range_from_rng {
    ($output_type: ty, $unsigned_type: ty, $generate_type: ty) => {
        impl RangeFromRng for $output_type {
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_possible_wrap,
                clippy::cast_lossless
            )]
            fn range_from_rng<T: Rng>(
                rng: &mut T,
                range: impl Into<GenerateRange<$output_type>>,
            ) -> Self {
                let GenerateRange {
                    start,
                    end_inclusive,
                } = range.into();
                if start == <$output_type>::MIN && end_inclusive == <$output_type>::MAX {
                    return rng.random::<$generate_type>() as $output_type;
                }
                assert!(start <= end_inclusive, "Inverted range");
                let span = (end_inclusive.wrapping_sub(start).wrapping_add(1)) as $unsigned_type;
                if span == 0 {
                    return start;
                }
                start.wrapping_add(
                    (<$generate_type>::zero_based_range_from_rng(rng, span as $generate_type)
                        as $output_type),
                )
            }
        }
    };
}

// We could have used u32 as the generated type here,
// which would probably perform marginally better.
// However, using u16 makes it much easier to test that
// the Lemire algorithm is correct and the distribution uniform.
range_from_rng! {u8, u8, u16}
range_from_rng! {i8, u8, u16}

range_from_rng! {u16, u16, u32}
range_from_rng! {i16, u16, u32}

range_from_rng! {u32, u32, u32}
range_from_rng! {i32, u32, u32}

range_from_rng! {u64, u64, u64}
range_from_rng! {i64, u64, u64}

range_from_rng! {u128, u128, u128}
range_from_rng! {i128, u128, u128}

#[cfg(target_pointer_width = "16")]
range_from_rng! {usize, usize, u32}
#[cfg(target_pointer_width = "32")]
range_from_rng! {usize, usize, u32}
#[cfg(target_pointer_width = "64")]
range_from_rng! {usize, usize, u32}

#[cfg(target_pointer_width = "16")]
range_from_rng! {isize, usize, u32}
#[cfg(target_pointer_width = "32")]
range_from_rng! {isize, usize, u32}
#[cfg(target_pointer_width = "64")]
range_from_rng! {isize, usize, u64}

impl RangeFromRng for f32 {
    #[allow(clippy::cast_precision_loss)]
    fn range_from_rng<T: Rng>(rng: &mut T, range: impl Into<GenerateRange<f32>>) -> Self {
        let GenerateRange {
            start,
            end_inclusive,
        } = range.into();
        let span = end_inclusive - start;

        // The simple algorith is just to generate an integer of the same size and convert it
        // to a float while scaling it.  However, this does not utilize the full dynamic range
        // of the mantissa when the integer is small.  The rand crate seems to do this.
        // An ideal algorithm should draw a real number, then round that to the nearest float
        // representation, in order to allow all possible float values to be possible outcomes.
        // This would be equivalent to drawing an int with virtually infinite size before
        // converting to float.
        // In practice, we just need enough bits to ensure that the mantissa is fully used.
        // A u64 will suffice, unless it has enough leading zero bits that there are less
        // than 24 remaining bits (because the mantissa has 23 bits plus an initial implicit 1).
        // We thus check the number of leading 0 bits, and draw one more random u64
        // to make the integer value u128 if necessary.
        // It is theoretically possible that a u128 this still not enough, but the probability
        // of that many leading zero bits is more than small enough to ignore.
        // Allways using u128 would be simpler, but not as fast.
        let r = rng.random::<u64>();
        let normalized = if (r >> 23) != 0 {
            (r as f32) / 2_f32.powi(64)
        } else {
            let r = (u128::from(r) << 64) | u128::from(rng.random::<u64>());
            (r as f32) / 2_f32.powi(128)
        };
        normalized * span + start
    }
}

impl RangeFromRng for f64 {
    #[allow(clippy::cast_precision_loss)]
    fn range_from_rng<T: Rng>(rng: &mut T, range: impl Into<GenerateRange<f64>>) -> Self {
        let GenerateRange {
            start,
            end_inclusive: end,
        } = range.into();
        let span = end - start;

        // The simple algorith is just to generate an integer of the same size and convert it
        // to a float while scaling it.  However, this does not utilize the full dynamic range
        // of the mantissa when the integer is small.  The rand crate seems to do this.
        // An ideal algorithm should draw a real number, then round that to the nearest float
        // representation, in order to allow all possible float values to be possible outcomes.
        // This would be equivalent to drawing an int with virtually infinite size before
        // converting to float.
        // In practice, we just need enough bits to ensure that the mantissa is fully used.
        // A u64 will suffice, unless it has enough leading zero bits that there are less
        // than 53 remaining bits (because the mantissa has 52 bits plus an initial implicit 1).
        // We thus check the number of leading 0 bits, and draw one more random u64
        // to make the integer value u128 if necessary.
        // It is theoretically possible that a u128 this still not enough, but the probability
        // of that many leading zero bits is more than small enough to ignore.
        // Allways using u128 would be simpler, but not as fast.
        let r = rng.random::<u64>();
        let normalized = if (r >> 52) != 0 {
            (r as f64) / 2_f64.powi(64)
        } else {
            let r = (u128::from(r) << 64) | u128::from(rng.random::<u64>());
            (r as f64) / 2_f64.powi(128)
        };
        normalized * span + start
    }
}

#[cfg(test)]
mod tests {
    use crate::rng::Rng;

    struct CountingRng(pub u64);

    impl CountingRng {
        fn new() -> Self {
            // Start near the max to ensure that the uniformity tests
            // hit the area where numbers must be discarded:
            Self(18446744073709550681)
        }
    }

    impl Rng for CountingRng {
        fn random_u32(&mut self) -> u32 {
            self.random_u64() as u32
        }

        fn random_u64(&mut self) -> u64 {
            let result = self.0;
            self.0 = self.0.wrapping_add(1);
            result
        }
    }

    #[test]
    fn test_range_u8_is_uniform() {
        const START: u8 = 13;
        const END: u8 = 42;
        const LEN: usize = (END - START) as usize;

        // u8 ranges are generated from u16 random values.
        // If we start the CountingRng at 0 and draw twice as many range values
        // as the Lemire algorithm can output for all possible u16 values,
        // then we will have tested all possible outcomes, and the distribution
        // should be uniform:
        let mut rng = CountingRng(0);
        let iterations: usize = 2 * ((1 << 16) / LEN) * LEN;

        let mut count: [usize; LEN] = [0; LEN];
        for _i in 0..iterations {
            let value = rng.range(START..END);
            assert!(value >= START);
            assert!(value < END);
            let inx = (value - START) as usize;
            count[inx] += 1;
        }
        for i in 0..LEN {
            assert_eq!(count[0], count[i]);
        }
    }

    #[test]
    fn test_range_i8_is_uniform() {
        const START: i8 = -127;
        const END: i8 = 126;
        const LEN: usize = ((END as isize) - (START as isize)) as usize;

        // i8 ranges are generated from u16 random values.
        // If we start the CountingRng at 0 and draw twice as many range values
        // as the Lemire algorithm can output for all possible u16 values,
        // then we will have tested all possible outcomes, and the distribution
        // should be uniform:
        let mut rng = CountingRng(0);
        let iterations: usize = 2 * ((1 << 16) / LEN) * LEN;

        let mut count: [usize; LEN] = [0; LEN];
        for _ in 0..iterations {
            let value = rng.range(START..END);
            assert!(value >= START);
            assert!(value < END);
            let inx = (value as isize).wrapping_sub(START as isize) as usize;
            count[inx] += 1;
        }
        for i in 0..LEN {
            assert_eq!(count[0], count[i]);
        }
    }

    #[test]
    fn test_unbounded_range_u8() {
        let mut rng = CountingRng::new();
        let mut count: [u8; 256] = [0; 256];
        for _ in 0..100 * 256 {
            let value: u8 = rng.range(..);
            count[value as usize] += 1;
        }
        for i in 0..256 {
            assert_eq!(count[0], count[i], "failed for {i}");
        }
    }

    #[test]
    fn test_range_boundaries() {
        let mut rng = CountingRng::new();
        let _: u8 = rng.range(0..=255);
        let _: u8 = rng.range(..=255);
        assert_eq!(255u8, rng.range(255u8..=255));
        assert_eq!(0u8, rng.range(0u8..=0));
    }

    struct CountingRng128 {
        next: u128,
        high: bool,
    }

    impl CountingRng128 {
        fn new() -> Self {
            // Start near the max to ensure that the uniformity tests
            // hit the area where numbers must be discarded:
            Self {
                next: u128::MAX - 100,
                high: true,
            }
        }
    }

    impl Rng for CountingRng128 {
        fn random_u32(&mut self) -> u32 {
            unimplemented!()
        }

        fn random_u64(&mut self) -> u64 {
            let random = if self.high {
                (self.next >> 64) as u64
            } else {
                let low = self.next as u64;
                self.next = self.next.wrapping_add(1);
                low
            };
            self.high = !self.high;
            random
        }
    }

    #[test]
    fn test_range_u128_is_uniform() {
        let mut rng = CountingRng128::new();
        const START: u128 = 13;
        const END: u128 = 42;
        const LEN: usize = (END - START) as usize;
        let mut count: [u8; LEN] = [0; LEN];
        for _ in 0..100 * LEN {
            let value = rng.range(START..END);
            assert!(value >= START);
            assert!(value < END);
            let inx = (value - START) as usize;
            count[inx] += 1;
        }
        for i in 0..LEN {
            assert_eq!(count[0], count[i]);
        }
    }

    struct FloatRangeGenerator(u128);

    impl FloatRangeGenerator {
        fn new(leading_zeros: usize) -> Self {
            Self(
                ((1_u128 << 127) >> leading_zeros)
                    | (0xDEADBEEFDEADBEEF0000000000000000_u128 >> (leading_zeros + 1)),
            )
        }
    }

    impl Rng for FloatRangeGenerator {
        fn random_u32(&mut self) -> u32 {
            0
        }

        fn random_u64(&mut self) -> u64 {
            let random = self.0 >> 64;
            self.0 = self.0 << 64;
            random as u64
        }
    }

    #[test]
    fn test_float_ranges() {
        for leading_zeros in 0..64 {
            let mut rng = FloatRangeGenerator::new(leading_zeros);
            let value: f64 = rng.range(0.0..1.0);
            let bytes = value.to_ne_bytes();

            // The algorithm should always fill the manitissa, so the "DEADBEEF" pattern
            // should always be in the same place:
            assert_eq!(bytes[5], 0xea);
            assert_eq!(bytes[4], 0xdb);
            assert_eq!(bytes[3], 0xee);
            assert_eq!(bytes[2], 0xfd);
        }
    }

    #[test]
    fn test_shuffle() {
        let mut rng = CountingRng::new();
        let mut numbers = vec![1, 2, 3, 4, 5];
        rng.shuffle(&mut numbers);
        assert_eq!(numbers, vec![5, 1, 2, 3, 4]);
    }

    #[test]
    fn test_shuffle_empty_slice() {
        let mut rng = CountingRng::new();
        let mut numbers: Vec<u8> = vec![];
        rng.shuffle(&mut numbers);
        assert_eq!(numbers, vec![]);
    }
}
