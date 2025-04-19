#![forbid(unsafe_code)]
#![allow(clippy::inline_always)]

#[cfg(all(unix, feature = "std"))]
pub use crate::devices::DevUrandom;
use crate::ranges::GenerateRange;
use crate::rngs::{RangeFromRng, ValueFromRng};
#[cfg(all(not(unix), feature = "std"))]
use crate::GetRandom;
use crate::{RandomDevice, Rng};

/// An xoshiro256++ 1.0 (see <https://prng.di.unimi.it>) random generator.
/// This is an efficient PRNG with good random properties, but not cryptographically secure:
/// An attacker will be able to calculate the internal state by observing
/// a number of samples, and thus predict future output.
#[allow(clippy::module_name_repetitions)]
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
    #[must_use]
    pub fn new() -> Self {
        #[cfg(unix)]
        let rng = Self::from_device(&mut DevUrandom::new());
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
    /// The `SplitMix64` algorithm is used to generate more seed bytes
    /// from the 64 bit seed value in order to initialize the whole state.
    ///
    /// # Arguments
    ///
    /// * `seed`: The seed to use
    ///
    /// returns: Xoshiro256pp
    #[must_use]
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
    pub fn range<T>(&mut self, range: impl Into<GenerateRange<T>>) -> T
    where
        T: RangeFromRng,
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
    pub fn iter<'a, T>(&'a mut self) -> impl Iterator<Item = T> + 'a
    where
        T: ValueFromRng + 'a,
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
    pub fn iter_u8(&mut self) -> impl Iterator<Item = u8> + '_
    where
        Self: Sized,
    {
        <Self as Rng>::iter_u8(self)
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
    /// let mut rng = smallrand::Xoshiro256pp::new();
    /// let mut data = [0_usize; 4];
    /// rng.fill(&mut data);
    /// }
    /// ```
    #[inline(always)]
    pub fn fill<T>(&mut self, destination: &mut [T])
    where
        T: ValueFromRng,
        Self: Sized,
    {
        <Self as Rng>::fill(self, destination);
    }

    /// Fills a mutable slice of u8 with random values.
    /// Faster than `fill` for u8 values.
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
    /// let mut rng = smallrand::Xoshiro256pp::new();
    /// let mut data = [0_u8; 4];
    /// rng.fill_u8(&mut data);
    /// }
    /// ```
    #[inline(always)]
    pub fn fill_u8(&mut self, destination: &mut [u8])
    where
        Self: Sized,
    {
        <Self as Rng>::fill_u8(self, destination);
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
    /// let mut rng = smallrand::Xoshiro256pp::new();
    /// let mut numbers = vec![1, 2, 3, 4, 5];
    /// rng.shuffle(&mut numbers);
    /// }
    /// ```
    #[inline(always)]
    pub fn shuffle<T>(&mut self, target: &mut [T])
    where
        T: Clone,
        Self: Sized,
    {
        <Self as Rng>::shuffle(self, target);
    }

    // This is "next" from the C reference implementation
    #[inline(always)]
    pub fn next_random(&mut self) -> u64 {
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

#[cfg(feature = "std")]
impl Default for Xoshiro256pp {
    fn default() -> Self {
        Self::new()
    }
}

impl Rng for Xoshiro256pp {
    #[allow(clippy::cast_possible_truncation)]
    #[inline(always)]
    fn random_u32(&mut self) -> u32 {
        self.random_u64() as u32
    }

    #[inline(always)]
    fn random_u64(&mut self) -> u64 {
        self.next_random()
    }
}

/// This is the `SplitMax` algorithm from <https://prng.di.unimi.it/splitmix64.c>
/// It is used here to generate more seed bytes from a 64 bit value.
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(state: u64) -> Self {
        Self { state }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}

#[cfg(test)]
mod tests {
    use super::Xoshiro256pp;

    struct DummyDevice;

    impl crate::RandomDevice for DummyDevice {
        fn seed_bytes<const N: usize>(&mut self) -> [u8; N] {
            core::array::from_fn(|i| (i + 42) as u8)
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
        let mut rng = Xoshiro256pp::from_seed(0u64);
        assert_eq!(rng.range(11_u8..42), 30)
    }

    #[test]
    fn xoshiro_generate_bools() {
        let mut rng = xoshiro();
        assert_eq!(
            vec![true, true, false, true, true, true],
            rng.iter().take(6).collect::<Vec<_>>()
        );
    }

    #[test]
    fn xoshiro_generate_u8() {
        let mut rng = xoshiro();
        assert_eq!(
            vec![93, 199, 18, 93, 255, 159],
            rng.iter().take(6).collect::<Vec<u8>>()
        );
    }

    #[test]
    fn xoshiro_generate_u8_fast() {
        let mut rng = xoshiro();
        assert_eq!(
            vec![93, 91, 89, 95, 93, 91],
            rng.iter_u8().take(6).collect::<Vec<_>>()
        );
    }

    #[test]
    fn xoshiro_generate_u16() {
        let mut rng = xoshiro();
        assert_eq!(
            vec![23389, 17863, 786, 12381, 18687, 18079],
            rng.iter().take(6).collect::<Vec<u16>>()
        );
    }

    #[test]
    fn xoshiro_generate_u32() {
        let mut rng = xoshiro();
        assert_eq!(
            vec![1599691613, 1187268039, 3807576850, 1187065949, 2131446015, 3237824159],
            rng.iter().take(6).collect::<Vec<u32>>()
        );
    }

    #[test]
    fn xoshiro_generate_u128() {
        let mut rng = xoshiro();
        assert_eq!(
            vec![
                116106803150699428516699394013734913479,
                216263115653590202844377321789695537245,
                328425740128965645684326511152092497567,
                254561204173543679954141383040234184739,
                40455170329874112030313783831106647436,
                309825739868400302084046781679576737863
            ],
            rng.iter().take(6).collect::<Vec<u128>>()
        );
    }

    #[test]
    fn xoshiro_generate_usize() {
        let mut rng = xoshiro();
        assert_eq!(
            vec![
                6294162410816756573,
                4666366678484141511,
                11723646991005319954,
                3577826909737791581,
                17803995047399213311,
                17636447047143736991
            ],
            rng.iter().take(6).collect::<Vec<usize>>()
        );
    }

    #[test]
    fn xoshiro_fill_u32() {
        let mut rng = xoshiro();
        let mut data = [0_u32; 4];
        rng.fill(&mut data);
        assert_eq!(&vec![1599691613, 1187268039, 3807576850, 1187065949], &data);
    }

    #[test]
    fn xoshiro_fill_u8() {
        let mut rng = xoshiro();
        let mut data = [0_u8; 4];
        rng.fill_u8(&mut data);
        assert_eq!(&vec![93, 91, 89, 95], &data);
    }

    #[test]
    fn xoshiro_bounded_range_f64() {
        let mut rng = xoshiro();
        let mut min = 42_f64;
        let mut max = 4_f64;
        for _ in 0..100 * 256 {
            let value: f64 = rng.range(4.0..42.0);
            assert!(value >= 4.0);
            assert!(value <= 42.0);
            min = min.min(value);
            max = max.max(value);
        }
        assert!(min < 4.01);
        assert!(max >= 41.99);
    }

    #[test]
    fn xoshiro_bounded_range_f32() {
        let mut rng = xoshiro();
        let mut min = 42_f32;
        let mut max = 4_f32;
        for _ in 0..100 * 256 {
            let value: f32 = rng.range(4.0..42.0);
            assert!(value >= 4.0);
            assert!(value <= 42.0);
            min = min.min(value);
            max = max.max(value);
        }
        assert!(min < 4.01);
        assert!(max >= 41.99);
    }
}
