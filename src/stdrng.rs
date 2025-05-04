#![forbid(unsafe_code)]

use crate::chacha::ChaCha12;
use crate::devices::RandomDevice;
use crate::ranges::GenerateRange;
use crate::rng::Rng;
use crate::rng::{RangeFromRng, ValueFromRng};
use crate::SecureDevice;

/// This is the default random generator. It has more state than [SmallRng](crate::SmallRng)
/// and is slower, but it has much better security properties.
/// The PRNG algorithm currently used is [ChaCha12], which is based on the
/// chacha crypto algorithm with 12 rounds.
///
/// This crypto algorithm is currently unbroken and can be used to implement
/// cryptographically secure random generators, but please note
/// that no guarantees of any kind are made that this particular implementation
/// is cryptographically secure.
///
/// The algorithm may change at any time, so if your
/// code depends on the algorithm staying the same then you should
/// use a specific algorithm instead.
///
/// Note that chacha is limited to generating 2^64 blocks (2^70 bytes).
/// The algorithm will panic if this limit is exceeded.
pub struct StdRng(Impl);

type Impl = ChaCha12;

impl Rng for StdRng {
    #[inline]
    fn random_u32(&mut self) -> u32 {
        self.0.random_u32()
    }

    #[inline]
    fn random_u64(&mut self) -> u64 {
        self.0.random_u64()
    }
}

impl StdRng {
    /// Creates a new random generator with a seed from a [SecureDevice].
    /// This type of device performs health tests on the system entropy source for extra security.
    ///
    /// returns: `StdRng`
    #[cfg(feature = "std")]
    #[must_use]
    pub fn new() -> Self {
        Self(Impl::from_device(&mut SecureDevice::new()))
    }

    /// Creates a new random generator with a seed from a [RandomDevice].
    /// Note that for uses that require security, it is recommended to
    /// use the `new` function instead, which uses a [SecureDevice] for entrpy.
    ///
    /// # Arguments
    ///
    /// * `random_device`: The device to get the seed from
    ///
    /// returns: `StdRng`
    pub fn from_device<T>(random_device: &mut T) -> Self
    where
        T: RandomDevice,
    {
        Self(Impl::from_device(random_device))
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
    /// let mut rng = smallrand::StdRng::new();
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
    /// let mut rng = smallrand::StdRng::new();
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
    /// let mut rng = smallrand::StdRng::new();
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
    /// let mut rng = smallrand::StdRng::new();
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
    /// let mut rng = smallrand::StdRng::new();
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
    /// let mut rng = smallrand::StdRng::new();
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
    /// let mut rng = smallrand::StdRng::new();
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
impl Default for StdRng {
    fn default() -> Self {
        Self::new()
    }
}
