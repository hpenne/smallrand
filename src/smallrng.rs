#![forbid(unsafe_code)]

use crate::entropy::EntropySource;
use crate::ranges::GenerateRange;
use crate::rng::Rng;
use crate::rng::{RangeFromRng, ValueFromRng};
use crate::xoshiro::Xoshiro256pp;
#[cfg(feature = "std")]
use crate::SecureEntropy;
use crate::SplitMix;

/// This is a numerically good PRNG if you need something small and fast
/// but not cryptographically secure.
/// The PRNG currently used is [Xoshiro256pp].
///
/// The algorithm may change at any time, so if your
/// code depends on the algorithm/output staying the same then you should
/// use a specific algorithm instead.
pub struct SmallRng(Impl);

type Impl = Xoshiro256pp;

impl Rng for SmallRng {
    #[inline]
    fn random_u32(&mut self) -> u32 {
        self.0.random_u32()
    }

    #[inline]
    fn random_u64(&mut self) -> u64 {
        self.0.random_u64()
    }
}

impl SmallRng {
    /// Creates a new random generator with a seed from a [DefaultEntropy].
    ///
    /// returns: `SmallRng`
    #[cfg(feature = "std")]
    #[must_use]
    pub fn new() -> Self {
        Self(Impl::from_entropy(&mut SecureEntropy::new()))
    }

    /// Creates a new random generator with a seed from an [EntropySource].
    ///
    /// # Arguments
    ///
    /// * `entropy_source`: The entropy source to get the seed from
    ///
    /// returns: [SmallRng]
    pub fn from_entropy<T>(entropy_source: &mut T) -> Self
    where
        T: EntropySource,
    {
        Self(Impl::from_entropy(entropy_source))
    }

    /// Creates a new random generator with a specified seed.
    ///
    /// WARNING: A single u64 is less entropy data than the RNG really needs.
    /// This function is only intended for testing where you want a fixed seed
    /// to generate the same output every time.
    /// You should use other functions to create the RNG in production code.
    ///
    /// # Arguments
    ///
    /// * `seed`: The seed to use
    ///
    /// returns: [SmallRng]
    ///
    /// # Examples
    /// ```
    /// let mut rng = smallrand::SmallRng::from_seed(42);
    /// let random_value : u32 = rng.random();
    /// ```
    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        Self(Impl::from_entropy(&mut SplitMix::new(seed)))
    }

    /// Generates a single random integer
    ///
    /// # Arguments
    ///
    /// returns: A random integer
    ///
    /// # Examples
    ///
    /// ```
    /// #[cfg(feature = "std")]
    /// {
    /// let mut rng = smallrand::SmallRng::new();
    /// let random_value : u32 = rng.random();
    /// }
    /// ```
    #[inline]
    pub fn random<T>(&mut self) -> T
    where
        T: ValueFromRng,
        Self: Sized,
    {
        self.0.random()
    }

    /// Generates a single random integer in a specified range.
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
    /// # Examples
    ///
    /// ```
    /// #[cfg(feature = "std")]
    /// {
    /// let mut rng = smallrand::SmallRng::new();
    /// let random_value : u32 = rng.range(..42);
    /// let float : f64 = rng.range::<f64>(1.0..42.0);
    /// }
    /// ```
    #[inline]
    pub fn range<T>(&mut self, range: impl Into<GenerateRange<T>>) -> T
    where
        T: RangeFromRng,
        Self: Sized,
    {
        self.0.range(range)
    }

    /// Provides an iterator that emits random values.
    ///
    /// returns: An iterator that outputs random values. Never None.
    ///
    /// # Examples
    ///
    /// ```
    /// #[cfg(feature = "std")]
    /// {
    /// let mut rng = smallrand::SmallRng::new();
    /// let random_values = rng.iter().take(10).collect::<Vec<u32>>();
    /// }
    /// ```
    #[inline]
    pub fn iter<'a, T>(&'a mut self) -> impl Iterator<Item = T> + 'a
    where
        T: ValueFromRng + 'a,
        Self: Sized,
    {
        self.0.iter()
    }

    /// Provides an iterator that emits random u8 values.
    /// Same as the generic variant, but more efficient.
    ///
    /// returns: An iterator that outputs random u8 values. Never None.
    ///
    /// # Examples
    ///
    /// ```
    /// #[cfg(feature = "std")]
    /// {
    /// let mut rng = smallrand::SmallRng::new();
    /// let random_values = rng.iter_u8().take(10).collect::<Vec<_>>();
    /// }
    /// ```
    #[inline]
    pub fn iter_u8(&mut self) -> impl Iterator<Item = u8> + '_
    where
        Self: Sized,
    {
        self.0.iter_u8()
    }

    /// Fills a mutable slice with random values.
    ///
    /// # Arguments
    ///
    /// * `destination`: The slice to fill
    ///
    /// # Examples
    ///
    /// ```
    /// #[cfg(feature = "std")]
    /// {
    /// let mut rng = smallrand::SmallRng::new();
    /// let mut data = [0_usize; 4];
    /// rng.fill(&mut data);
    /// }
    /// ```
    #[inline]
    pub fn fill<T>(&mut self, destination: &mut [T])
    where
        T: ValueFromRng,
        Self: Sized,
    {
        self.0.fill(destination);
    }

    /// Fills a mutable slice of u8 with random values.
    /// Faster than [fill](Self::fill()) for u8 values.
    ///
    /// # Arguments
    ///
    /// * `destination`: The slice to fill
    ///
    /// # Examples
    ///
    /// ```
    /// #[cfg(feature = "std")]
    /// {
    /// let mut rng = smallrand::SmallRng::new();
    /// let mut data = [0_u8; 4];
    /// rng.fill_u8(&mut data);
    /// }
    /// ```
    #[inline]
    pub fn fill_u8(&mut self, destination: &mut [u8])
    where
        Self: Sized,
    {
        self.0.fill_u8(destination);
    }

    /// Shuffles the elements of a slice
    ///
    /// # Arguments
    ///
    /// * `target`: The slice to shuffle
    ///
    /// # Examples
    ///
    /// ```
    /// #[cfg(feature = "std")]
    /// {
    /// let mut rng = smallrand::SmallRng::new();
    /// let mut numbers = vec![1, 2, 3, 4, 5];
    /// rng.shuffle(&mut numbers);
    /// }
    /// ```
    #[inline]
    pub fn shuffle<T>(&mut self, target: &mut [T])
    where
        T: Clone,
        Self: Sized,
    {
        self.0.shuffle(target);
    }
}

#[cfg(feature = "std")]
impl Default for SmallRng {
    fn default() -> Self {
        Self::new()
    }
}
