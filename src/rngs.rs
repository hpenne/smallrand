#![forbid(unsafe_code)]
#![allow(clippy::inline_always)]

use core::ops::Bound;
use core::ops::RangeBounds;

/// This trait that all PRNGs must implement.
/// It defines default implementations of functions
/// to be supported by all PRNGs,
/// as well the declarations of two internal helper
/// functions that provide values to these functions.
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
    #[inline(always)]
    fn random<T>(&mut self) -> T
    where
        T: ValueFromRng,
        Self: Sized,
    {
        T::value_from_rng(self)
    }

    /// Generates a single random integer in a specified range.
    /// The distribution is strictly uniform.
    ///
    /// # Arguments
    ///
    /// * `range`: The range of the uniform distribution.
    ///
    /// returns: A random integer
    ///
    fn range<T, R>(&mut self, range: R) -> T
    where
        T: RangeFromRng,
        R: RangeBounds<T>,
        Self: Sized,
    {
        T::range_from_rng(self, range)
    }

    /// Provides an iterator that emits random values.
    ///
    /// returns: An iterator that outputs random values. Never None.
    ///
    #[inline(always)]
    fn iter<T>(&mut self) -> impl Iterator<Item = T>
    where
        T: ValueFromRng,
        Self: Sized,
    {
        core::iter::from_fn(|| Some(self.random()))
    }

    /// Provides an iterator that emits random u8 values.
    /// Same as the generic variant, but more efficient.
    ///
    /// returns: An iterator that outputs random u8 values. Never None.
    ///
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
    fn shuffle<T>(&mut self, target: &mut [T])
    where
        T: Clone,
        Self: Sized,
    {
        // This is the forward version of the Fisher-Yates/Knuth shuffle:
        // https://en.wikipedia.org/wiki/Fisher–Yates_shuffle
        if !target.is_empty() {
            for inx in 0..target.len().wrapping_sub(1) {
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
    fn range_from_rng<T: Rng, R: RangeBounds<Self>>(device: &mut T, range: R) -> Self;
}

macro_rules! range_from_rng {
    ($output_type: ty) => {
        fn range_from_rng<T: Rng, R: RangeBounds<$output_type>>(device: &mut T, range: R) -> Self {
            let start: $output_type = match range.start_bound() {
                Bound::Included(start) => *start,
                Bound::Excluded(start) => start.checked_add(1).expect("Range start overflow"),
                Bound::Unbounded => 0,
            };
            let span: $output_type = match range.end_bound() {
                Bound::Unbounded | Bound::Included(&<$output_type>::MAX) if start == 0 => {
                    return device.random();
                }
                Bound::Included(&<$output_type>::MAX) => <$output_type>::MAX - start + 1,
                Bound::Included(end) => (end + 1)
                    .checked_sub(start)
                    .expect("Range end before start"),
                Bound::Excluded(end) => end.checked_sub(start).expect("Range end before start"),
                Bound::Unbounded => <$output_type>::MAX - start + 1,
            };
            if span == 0 {
                return start;
            }
            let mut random_value: $output_type = device.random();
            let reduced_max = <$output_type>::MAX - span + 1;
            let max_valid_value = <$output_type>::MAX - (reduced_max % span);
            while random_value > max_valid_value {
                random_value = device.random();
            }
            start + (random_value % span)
        }
    };
}

impl RangeFromRng for u8 {
    range_from_rng! {u8}
}

impl RangeFromRng for u16 {
    range_from_rng! {u16}
}

impl RangeFromRng for u32 {
    range_from_rng! {u32}
}

impl RangeFromRng for u64 {
    range_from_rng! {u64}
}

impl RangeFromRng for u128 {
    range_from_rng! {u128}
}

impl RangeFromRng for usize {
    range_from_rng! {usize}
}

#[cfg(test)]
mod tests {
    use crate::rngs::Rng;

    struct CountingRng {
        next: u32,
    }

    impl CountingRng {
        fn new() -> Self {
            Self { next: 0 }
        }
    }

    impl Rng for CountingRng {
        fn random_u32(&mut self) -> u32 {
            let result = self.next;
            self.next = self.next + 1;
            result
        }

        fn random_u64(&mut self) -> u64 {
            self.random_u32() as u64
        }
    }

    #[test]
    fn test_range_u8_is_uniform() {
        let mut rng = CountingRng::new();
        const START: u8 = 13;
        const END: u8 = 42;
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
    fn test_shuffle() {
        let mut rng = CountingRng::new();
        let mut numbers = vec![1, 2, 3, 4, 5];
        rng.shuffle(&mut numbers);
        assert_eq!(numbers, vec![1, 3, 5, 2, 4]);
    }

    #[test]
    fn test_shuffle_empty_slice() {
        let mut rng = CountingRng::new();
        let mut numbers: Vec<u8> = vec![];
        rng.shuffle(&mut numbers);
        assert_eq!(numbers, vec![]);
    }
}
