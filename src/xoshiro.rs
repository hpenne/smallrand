#[cfg(all(unix, feature = "std"))]
pub use crate::devices::DevRandom;
use crate::rngs::{RangeFromRng, ValueFromRng};
#[cfg(all(not(unix), feature = "std"))]
use crate::GetRandom;
use crate::{RandomDevice, Rng};
use std::ops::RangeBounds;

/// An xoshiro256++ 1.0 (see <https://prng.di.unimi.it>) random generator.
/// This is an efficient PRNG with good random properties, but not cryptographically secure:
/// An attacker will be able to calculate the internal state by observing
/// a number of samples, and thus predict future output.
pub struct Xoshiro256pp {
    state: [u64; 4],
}

impl Xoshiro256pp {
    /// Creates a new xoshiro256++ random generator with a seed from a random device.
    ///
    /// # Arguments
    ///
    /// * `random_device`: The device to get the seed from
    ///
    /// returns: Xoshiro256pp
    #[cfg(feature = "std")]
    pub fn new() -> Self {
        #[cfg(unix)]
        let rng = Self::from_device(&mut DevRandom::new());
        #[cfg(not(unix))]
        let rng = Self::from_device(&mut GetRandom::new());
        rng
    }

    /// Creates a new xoshiro256++ random generator with a seed from a random device.
    ///
    /// # Arguments
    ///
    /// * `random_device`: The device to get the seed from
    ///
    /// returns: Xoshiro256pp
    pub fn from_device<T>(random_device: &mut T) -> Self
    where
        T: RandomDevice,
    {
        Self {
            state: core::array::from_fn(|_| random_device.seed::<u64>()),
        }
    }

    /// Creates a new xoshiro256++ random generator with a specified seed.
    ///
    /// # Arguments
    ///
    /// * `seed`: The seed to use
    ///
    /// returns: Xoshiro256pp
    pub fn from_seed(seed: u64) -> Self {
        let mut seed_generator = SplitMix64::new(seed);
        Self {
            state: core::array::from_fn(|_| seed_generator.next()),
        }
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
    /// let mut rng = smallrand::Xoshiro256pp::new();
    /// let random_value : u32 = rng.random();
    /// }
    /// ```
    #[inline(always)]
    pub fn random<T>(&mut self) -> T
    where
        T: ValueFromRng,
        Self: Sized,
    {
        <Self as Rng>::random(self)
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
    /// # Examples
    ///
    /// ```
    /// #[cfg(feature = "std")]
    /// {
    /// let mut rng = smallrand::Xoshiro256pp::new();
    /// let random_value : u32 = rng.range(..42);
    /// }
    /// ```
    #[inline(always)]
    pub fn range<T, R>(&mut self, range: R) -> T
    where
        T: RangeFromRng,
        R: RangeBounds<T>,
        Self: Sized,
    {
        <Self as Rng>::range(self, range)
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
    /// let mut rng = smallrand::Xoshiro256pp::new();
    /// let random_values = rng.iter().take(10).collect::<Vec<u32>>();
    /// }
    /// ```
    #[inline(always)]
    pub fn iter<T>(&mut self) -> impl Iterator<Item = T> + use<'_, T>
    where
        T: ValueFromRng,
        Self: Sized,
    {
        <Self as Rng>::iter(self)
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
    /// let mut rng = smallrand::Xoshiro256pp::new();
    /// let random_values = rng.iter_u8().take(10).collect::<Vec<_>>();
    /// }
    /// ```
    #[inline(always)]
    pub fn iter_u8(&mut self) -> impl Iterator<Item = u8> + use<'_>
    where
        Self: Sized,
    {
        <Self as Rng>::iter_u8(self)
    }

    // This is "next" from the C reference implementation
    pub fn next(&mut self) -> u64 {
        let result = (self.state[0].wrapping_add(self.state[3]))
            .rotate_left(23)
            .wrapping_add(self.state[0]);

        let t = self.state[1] << 17;

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];

        self.state[2] ^= t;

        self.state[3] = self.state[3].rotate_left(45);

        result
    }
}

impl Default for Xoshiro256pp {
    fn default() -> Self {
        Self::new()
    }
}
impl Rng for Xoshiro256pp {
    fn random_u32(&mut self) -> u32 {
        self.random_u64() as u32
    }

    fn random_u64(&mut self) -> u64 {
        self.next()
    }
}

pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(state: u64) -> Self {
        Self { state }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyDevice;

    impl RandomDevice for DummyDevice {
        fn seed_bytes<const N: usize>(&mut self) -> [u8; N] {
            [42; N]
        }
    }

    fn xoshiro() -> Xoshiro256pp {
        Xoshiro256pp::from_device(&mut DummyDevice {})
    }

    #[test]
    fn test_xoshiro_output() {
        // These test vectors match the values generated by the `rand` crate:
        let mut rng = Xoshiro256pp {
            state: [1, 2, 3, 4],
        };
        assert_eq!(
            vec![
                41943041,
                58720359,
                3588806011781223,
                3591011842654386,
                9228616714210784205,
                9973669472204895162,
                14011001112246962877,
                12406186145184390807,
                15849039046786891736,
                10450023813501588000,
            ],
            rng.iter().take(10).collect::<Vec<u64>>()
        );
    }

    #[test]
    fn test_xoshiro_from_seed() {
        // These test vectors match the values generated by the `rand` crate:
        let mut rng = Xoshiro256pp::from_seed(0u64);
        assert_eq!(
            vec![
                5987356902031041503,
                7051070477665621255,
                6633766593972829180,
                211316841551650330,
                9136120204379184874,
                379361710973160858,
                15813423377499357806,
                15596884590815070553,
                5439680534584881407,
                1369371744833522710,
            ],
            rng.iter().take(10).collect::<Vec<u64>>()
        );
    }

    #[test]
    fn test_xoshiro_random() {
        // These test vectors match the values generated by the `rand` crate:
        let mut rng = Xoshiro256pp::from_seed(0u64);
        assert_eq!(rng.random::<u64>(), 5987356902031041503)
    }

    #[test]
    fn test_xoshiro_range() {
        // These test vectors match the values generated by the `rand` crate:
        let mut rng = Xoshiro256pp::from_seed(0u64);
        assert_eq!(rng.range(11_u8..42), 17)
    }

    #[test]
    fn xoshiro_generate_bools() {
        let mut rng = xoshiro();
        assert_eq!(
            vec![false, true, false, false, true, true],
            rng.iter().take(6).collect::<Vec<_>>()
        );
    }

    #[test]
    fn xoshiro_generate_u8() {
        let mut rng = xoshiro();
        assert_eq!(
            vec![84, 63, 162, 30, 185, 255],
            rng.iter().take(6).collect::<Vec<u8>>()
        );
    }

    #[test]
    fn xoshiro_generate_u8_fast() {
        let mut rng = xoshiro();
        assert_eq!(
            vec![84, 84, 84, 84, 84, 84],
            rng.iter_u8().take(6).collect::<Vec<_>>()
        );
    }

    #[test]
    fn xoshiro_generate_u16() {
        let mut rng = xoshiro();
        assert_eq!(
            vec![21588, 16191, 41634, 8734, 45497, 1535],
            rng.iter().take(6).collect::<Vec<u16>>()
        );
    }

    #[test]
    fn xoshiro_generate_u32() {
        let mut rng = xoshiro();
        assert_eq!(
            vec![1414812756, 1061109567, 2728567458, 3299746334, 3115430329, 2173044223],
            rng.iter().take(6).collect::<Vec<u32>>()
        );
    }

    #[test]
    fn xoshiro_generate_u128() {
        let mut rng = xoshiro();
        assert_eq!(
            vec![
                112093014985720905609945432936159330111,
                216179386043890317965546119157345296926,
                122103223737332025759166659050613376511,
                28681241608870936069189409415718651905,
                270164761497474401175398671158277407014,
                107703748537752760609113074678511574743
            ],
            rng.iter().take(6).collect::<Vec<u128>>()
        );
    }
}
