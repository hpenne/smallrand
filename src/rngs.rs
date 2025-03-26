use crate::devices::RandomDevice;
use std::collections::Bound;
use std::ops::RangeBounds;

pub trait Rng {
    fn random_u32(&mut self) -> u32;

    fn random_u64(&mut self) -> u64;

    fn random<T>(&mut self) -> T
    where
        T: ValueFromRng,
        Self: Sized,
    {
        T::value_from_rng(self)
    }

    fn range<T, R>(&mut self, range: R) -> T
    where
        T: RangeFromRng,
        R: RangeBounds<T>,
        Self: Sized,
    {
        T::range_from_rng(self, range)
    }

    #[cfg(feature = "std")]
    fn iter<T>(&mut self) -> impl Iterator<Item = T>
    where
        T: ValueFromRng,
        Self: Sized,
    {
        std::iter::from_fn(|| Some(self.random()))
    }

    #[cfg(feature = "std")]
    fn iter_u8(&mut self) -> impl Iterator<Item = u8>
    where
        Self: Sized,
    {
        self.iter::<u64>().flat_map(|r| r.to_ne_bytes())
    }
}

pub fn random<T, R>(rng: &mut R) -> T
where
    T: ValueFromRng,
    R: Rng,
{
    T::value_from_rng(rng)
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
    fn value_from_rng<T: Rng>(rng: &mut T) -> Self {
        rng.random_u32() as Self
    }
}

impl ValueFromRng for u16 {
    fn value_from_rng<T: Rng>(rng: &mut T) -> Self {
        rng.random_u32() as Self
    }
}

impl ValueFromRng for u32 {
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
        (rng.random_u64() as u128) << 64 | (rng.random_u64() as u128)
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
                Bound::Excluded(start) => start + 1,
                Bound::Unbounded => 0,
            };
            let span: $output_type = match range.end_bound() {
                Bound::Unbounded | Bound::Included(&<$output_type>::MAX) if start == 0 => {
                    return device.random();
                }
                Bound::Included(end) => end - start + 1,
                Bound::Excluded(end) => end - start,
                Bound::Unbounded => <$output_type>::MAX - start + 1,
            };
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
        rng.state += seed;
        rng.next();
        rng
    }

    pub fn new<T>(random_device: &mut T) -> Self
    where
        T: RandomDevice,
    {
        Self::new_with_increment(random_device.seed(), Self::PCG_DEFAULT_INCREMENT_128)
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
    use crate::devices::RandomDevice;
    use crate::rngs::PcgXsl128_64;
    use crate::rngs::{random, Rng};

    struct DummyDevice;

    impl RandomDevice for DummyDevice {
        fn seed_bytes<const N: usize>(&mut self) -> [u8; N] {
            [42; N]
        }
    }

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
            assert_eq!(r, random(&mut rng));
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

    #[test]
    fn test_range_u8() {
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
}
