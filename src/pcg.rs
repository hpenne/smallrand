use crate::{RandomDevice, Rng, ValueFromRng};
use rand::random;

// This is a PCG (https://www.pcg-random.org) PRNG (specifically the "pcg_state_setseq_128")
pub struct PcgXsl128_64 {
    state: u128,
    inc: u128,
}

impl PcgXsl128_64 {
    const PCG_DEFAULT_MULTIPLIER_128: u128 = 0x2360_ED05_1FC6_5DA4_4385_DF64_9FCC_F645_u128;
    const PCG_DEFAULT_INCREMENT_128: u128 = 0x5851_F42D_4C95_7F2D_1405_7B7E_F767_814F;

    // This is "pcg_setseq_128_srandom_r" from the C reference implementation
    pub fn new_with_increment(seed: u128, inc: u128) -> Self {
        let mut rng = Self {
            state: 0,
            inc: (inc << 1) | 1,
        };
        rng.next();
        rng.state = rng.state.wrapping_add(seed);
        rng.next();
        rng
    }

    /// Creates a new PCG random generator with a seed from a random device.
    ///
    /// # Arguments
    ///
    /// * `random_device`: The device to get the seed from
    ///
    /// returns: PcgXsl128_64
    pub fn new<T>(random_device: &mut T) -> Self
    where
        T: RandomDevice,
    {
        Self::new_with_increment(random_device.seed(), Self::PCG_DEFAULT_INCREMENT_128)
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
    /// #[cfg(feature = "getrandom")]
    /// {
    /// let mut rng = urng::PcgXsl128_64::new(&mut urng::GetRandom::new());
    /// let random_value : u32 = rng.random();
    /// }
    /// ```
    pub fn random<T>(&mut self) -> T
    where
        T: ValueFromRng,
        Self: Sized,
    {
        <Self as Rng>::random(self)
    }

    // This is "pcg_setseq_128_step_r" from the C reference implementation
    fn next(&mut self) {
        self.state = self
            .state
            .wrapping_mul(Self::PCG_DEFAULT_MULTIPLIER_128)
            .wrapping_add(self.inc)
    }

    // This is "pcg_output_xsl_rr_128_64" from the C reference implementation
    fn output(&self) -> u64 {
        (((self.state >> 64) as u64) ^ (self.state as u64)).rotate_right((self.state >> 122) as u32)
    }
}

impl Rng for PcgXsl128_64 {
    fn random_u32(&mut self) -> u32 {
        self.random_u64() as u32
    }

    fn random_u64(&mut self) -> u64 {
        self.next();
        self.output()
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

    fn pcg() -> PcgXsl128_64 {
        PcgXsl128_64::new(&mut DummyDevice {})
    }

    #[test]
    fn test_pcg_output() {
        // These test vectors are from the PCG reference implementation:
        let mut rng = PcgXsl128_64::new_with_increment(42, 54);
        assert_eq!(
            vec![
                0x86b1da1d72062b68_u64,
                0x1304aa46c9853d39,
                0xa3670e9e0dd50358,
                0xf9090e529a7dae00,
                0xc85b9fd837996f2c,
                0x606121f8e3919196,
            ],
            rng.iter().take(6).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_pcg_output_with_default_increment() {
        let reference = vec![
            6238911245709606319_u64,
            5238302247168832727,
            4297377549515893626,
            7003277208431798990,
            10313739050882324746,
            7614407471834827897,
        ];
        let mut rng = pcg();
        assert_eq!(reference, rng.iter().take(6).collect::<Vec<_>>());
        rng = pcg();
        for r in reference {
            assert_eq!(r, rng.random());
        }
    }

    #[test]
    fn pcg_generate_bools() {
        let mut rng = pcg();
        assert_eq!(
            vec![true, true, false, false, false, true],
            rng.iter().take(6).collect::<Vec<_>>()
        );
    }

    #[test]
    fn pcg_generate_u8() {
        let mut rng = pcg();
        assert_eq!(
            vec![175_u8, 215, 122, 206, 10, 121],
            rng.iter().take(6).collect::<Vec<_>>()
        );
    }

    #[test]
    fn pcg_generate_u8_fast() {
        let mut rng = pcg();
        assert_eq!(
            vec![175, 253, 181, 9, 185, 16],
            rng.iter_u8().take(6).collect::<Vec<_>>()
        );
    }

    #[test]
    fn pcg_generate_u16() {
        let mut rng = pcg();
        assert_eq!(
            vec![64943_u16, 62679, 11130, 43726, 22794, 35961],
            rng.iter().take(6).collect::<Vec<_>>()
        );
    }

    #[test]
    fn pcg_generate_u32() {
        let mut rng = pcg();
        assert_eq!(
            vec![
                162921903_u32,
                3701535959,
                3768724346,
                539667150,
                2984925450,
                3050015865
            ],
            rng.iter().take(6).collect::<Vec<_>>()
        );
    }

    #[test]
    fn pcg_generate_u128() {
        let mut rng = pcg();
        assert_eq!(
            vec![
                115087599048193556605888723764539094231_u128,
                79272623844024785956538378246444198606,
                190254904714650299646906997679195917433,
                195815291615320330060185591365323202247,
                266585136419131235011947267918666247456,
                314161403799413480381367577600824184889
            ],
            rng.iter().take(6).collect::<Vec<_>>()
        );
    }
}
