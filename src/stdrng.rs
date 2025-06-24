use crate::chacha::ChaCha12;
use crate::entropy::EntropySource;
use crate::ranges::GenerateRange;
use crate::rng::Rng;
use crate::rng::{RangeFromRng, ValueFromRng};
#[cfg(feature = "std")]
use crate::SecureEntropy;

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
    /// Creates a new random generator with a seed from a [SecureEntropy].
    /// This type of entropy source performs health tests on the system entropy source for extra security.
    /// Note that [SecureEntropy] has a small false alarm rate, which can cause intermittent fuzz test errors
    /// if a new [StdRng] instance is created for each fuzz test vector.
    /// If you need to avoid this then create your instances with `from_entropy` and pass in a [DefaultEntropy] instead.
    ///
    /// returns: `StdRng`
    #[cfg(feature = "std")]
    #[must_use]
    pub fn new() -> Self {
        Self(Impl::from_entropy(&mut SecureEntropy::new()))
    }

    /// Creates a new random generator with a seed from an [EntropySource].
    /// Note that for uses that require security, it is recommended to
    /// use the `new` function instead, which uses a [SecureEntropy] for entrpy.
    ///
    /// # Arguments
    ///
    /// * `entropy_source`: The entropy source to get the seed from
    ///
    /// returns: `StdRng`
    pub fn from_entropy<T>(entropy_source: &mut T) -> Self
    where
        T: EntropySource,
    {
        Self(Impl::from_entropy(entropy_source))
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

    pub fn from_entropy_and_nonce<T>(entropy_source: &mut T, nonce: [u8; 8]) -> Self
    where
        T: EntropySource,
    {
        Self(Impl::from_entropy_and_nonce(entropy_source, nonce))
    }
}

/// Creates a new random generator with a seed from a [SecureEntropy].
/// This type of entropy source performs health tests on the system entropy source for extra security.
/// Note that [SecureEntropy] has a small false alarm rate, which can cause intermittent fuzz test errors
/// if a new [StdRng] instance is created for each fuzz test vector.
/// If you need to avoid this then create your instances with `from_entropy` and pass in a [DefaultEntropy] instead.
#[cfg(feature = "std")]
impl Default for StdRng {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Rng, SplitMix, StdRng};

    #[test]
    fn test_forwarding() {
        // Test that call forwarding isn't completely broken
        // (the forwarded-to functions are tested in chacha.rs)
        let mut rng =
            StdRng::from_entropy_and_nonce(&mut SplitMix::new(12345678), [1, 2, 3, 4, 5, 6, 7, 8]);
        assert_ne!(rng.random_u32(), rng.random_u32());
        assert_ne!(rng.random_u64(), rng.random_u64());
        assert_ne!(rng.random::<u16>(), rng.random::<u16>());
        assert_ne!(rng.random::<u32>(), rng.random::<u32>());
        assert_ne!(rng.random::<u64>(), rng.random::<u64>());

        assert_ne!(rng.range::<u32>(0..12452), rng.range::<u32>(0..12452));

        {
            let mut i = rng.iter::<u128>();
            i.next();
            assert_ne!(i.next(), i.next());
        }

        let mut a1 = [0_u8; 32];
        let mut a2 = [0_u8; 32];
        rng.fill(&mut a1);
        rng.fill(&mut a2);
        assert_ne!(a1, a2);

        a2 = a1;
        rng.fill_u8(&mut a1);
        rng.fill_u8(&mut a2);
        assert_ne!(a1, a2);

        a2 = a1;
        rng.shuffle(&mut a2);
        assert_ne!(a1, a2);
    }

    #[test]
    fn same_entropy_and_nonce_generates_same_values() {
        let mut rng1 =
            StdRng::from_entropy_and_nonce(&mut SplitMix::new(12345678), [1, 2, 3, 4, 5, 6, 7, 8]);
        let mut rng2 =
            StdRng::from_entropy_and_nonce(&mut SplitMix::new(12345678), [1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(rng1.random_u64(), rng2.random_u64());
    }

    #[test]
    fn same_entropy_and_different_nonce_produces_different_values() {
        let mut rng1 = StdRng::from_entropy(&mut SplitMix::new(12345678));
        let mut rng2 = StdRng::from_entropy(&mut SplitMix::new(12345678));
        assert_ne!(rng1.random_u64(), rng2.random_u64());
    }

    #[test]
    fn different_entropy_and_same_nonce_produces_different_values() {
        let mut rng1 =
            StdRng::from_entropy_and_nonce(&mut SplitMix::new(12345678), [1, 2, 3, 4, 5, 6, 7, 8]);
        let mut rng2 =
            StdRng::from_entropy_and_nonce(&mut SplitMix::new(87654321), [1, 2, 3, 4, 5, 6, 7, 8]);
        assert_ne!(rng1.random_u64(), rng2.random_u64());
    }
}
