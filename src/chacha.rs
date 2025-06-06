#![allow(clippy::inline_always)]

use crate::{nonces, EntropySource, Rng};
use std::ops::BitXor;

#[allow(clippy::doc_markdown)]
/// This is a random generator based on the ChaCha crypto algorithm with 12 rounds.
///
/// This algorithm is currently unbroken and can be used to implement
/// cryptographically secure random generators, but please note
/// that no guarantees of any kind are made that this particular implementation
/// is cryptographically secure.
///
/// Note that ChaCha is limited to generating 2^64 blocks (2^70 bytes).
/// The algorithm will panic if this limit is exceeded.
#[allow(clippy::module_name_repetitions)]
pub struct ChaCha12(ChaCha<12>);

impl ChaCha12 {
    /// Creates a new [ChaCha12] random generator using a seed from an [EntropySource].
    /// The nonce is taken from the nanoseconds part of `SystemTime` when
    /// building with `std` enabled, to provide an extra safety net in case the random
    /// entropy source is broken.
    /// For non-std builds, the nonce is 0 (which is what `rand` always does).
    ///
    /// # Arguments
    ///
    /// * `entropy_source`: The source of the seed
    ///
    /// returns: [ChaCha12]
    ///
    pub fn from_entropy<T>(entropy_source: &mut T) -> Self
    where
        T: EntropySource,
    {
        let mut key = [0; 32];
        entropy_source.fill(&mut key);
        Self(ChaCha::<12>::new(&key, nonces::nonce_u64()))
    }

    /// Creates a new [ChaCha12] random generator from a specified seed and nonce.
    ///
    /// # Arguments
    ///
    /// * `seed`: The seed (i.e. key) to initialize with
    /// * `nonce`: The nonce to initialize with
    ///
    /// returns: [ChaCha12]
    ///
    #[must_use]
    pub fn from_seed(seed: &[u8; 32], nonce: [u8; 8]) -> Self {
        Self(ChaCha::<12>::new(seed, nonce))
    }

    pub(crate) fn from_entropy_and_nonce<T>(entropy_source: &mut T, nonce: [u8; 8]) -> Self
    where
        T: EntropySource,
    {
        let mut key = [0; 32];
        entropy_source.fill(&mut key);
        Self(ChaCha::<12>::new(&key, nonce))
    }
}

impl Rng for ChaCha12 {
    #[inline(always)]
    fn random_u32(&mut self) -> u32 {
        self.0.random_u32()
    }

    #[inline(always)]
    fn random_u64(&mut self) -> u64 {
        self.0.random_u64()
    }

    #[inline(always)]
    fn fill_u8(&mut self, destination: &mut [u8]) {
        self.0.fill_u8(destination);
    }
}

struct ChaCha<const ROUNDS: usize> {
    state: [u32; 16],
    inx: usize,
    buffer: [u8; 64],
}

impl<const ROUNDS: usize> ChaCha<ROUNDS> {
    fn new(key: &[u8; 32], nonce: [u8; 8]) -> Self {
        const SIGMA: &[u8; 16] = b"expand 32-byte k";
        // The unwraps below cannot fail and will get optimized away
        let mut s = Self {
            state: [
                u32::from_le_bytes(SIGMA[0..4].try_into().unwrap()),
                u32::from_le_bytes(SIGMA[4..8].try_into().unwrap()),
                u32::from_le_bytes(SIGMA[8..12].try_into().unwrap()),
                u32::from_le_bytes(SIGMA[12..16].try_into().unwrap()),
                u32::from_le_bytes(key[0..4].try_into().unwrap()),
                u32::from_le_bytes(key[4..8].try_into().unwrap()),
                u32::from_le_bytes(key[8..12].try_into().unwrap()),
                u32::from_le_bytes(key[12..16].try_into().unwrap()),
                u32::from_le_bytes(key[16..20].try_into().unwrap()),
                u32::from_le_bytes(key[20..24].try_into().unwrap()),
                u32::from_le_bytes(key[24..28].try_into().unwrap()),
                u32::from_le_bytes(key[28..32].try_into().unwrap()),
                0,
                0,
                u32::from_le_bytes(nonce[0..4].try_into().unwrap()),
                u32::from_le_bytes(nonce[4..8].try_into().unwrap()),
            ],
            inx: 0,
            buffer: [0; 64],
        };
        s.generate_block();
        s
    }

    fn generate_block(&mut self) {
        let mut x = [0_u32; 16];
        x.copy_from_slice(&self.state);
        for _round in (0..ROUNDS).step_by(2) {
            // Odd round
            Self::quarter_round(&mut x, 0, 4, 8, 12);
            Self::quarter_round(&mut x, 1, 5, 9, 13);
            Self::quarter_round(&mut x, 2, 6, 10, 14);
            Self::quarter_round(&mut x, 3, 7, 11, 15);

            // Even round
            Self::quarter_round(&mut x, 0, 5, 10, 15);
            Self::quarter_round(&mut x, 1, 6, 11, 12);
            Self::quarter_round(&mut x, 2, 7, 8, 13);
            Self::quarter_round(&mut x, 3, 4, 9, 14);
        }
        for (i, element) in x.iter_mut().enumerate() {
            *element = element.wrapping_add(self.state[i]);
        }
        let mut blocks = self.buffer.chunks_exact_mut(core::mem::size_of::<u32>());
        for (i, block) in blocks.by_ref().enumerate() {
            block.copy_from_slice(&x[i].to_le_bytes());
        }
        self.state[12] = self.state[12].wrapping_add(1);
        if self.state[12] == 0 {
            self.state[13] = self.state[13]
                .checked_add(1)
                .expect("Max number of blocks exceeded");
        }
    }

    #[allow(clippy::many_single_char_names)]
    #[inline(always)]
    fn quarter_round(x: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
        x[a] = x[a].wrapping_add(x[b]);
        x[d] = x[d].bitxor(x[a]).rotate_left(16);
        x[c] = x[c].wrapping_add(x[d]);
        x[b] = x[b].bitxor(x[c]).rotate_left(12);
        x[a] = x[a].wrapping_add(x[b]);
        x[d] = x[d].bitxor(x[a]).rotate_left(8);
        x[c] = x[c].wrapping_add(x[d]);
        x[b] = x[b].bitxor(x[c]).rotate_left(7);
    }
}

impl<const ROUNDS: usize> Rng for ChaCha<ROUNDS> {
    #[inline]
    fn random_u32(&mut self) -> u32 {
        const SIZE: usize = core::mem::size_of::<u32>();
        if self.inx + SIZE > self.buffer.len() {
            self.generate_block();
            self.inx = 0;
        }
        let value = u32::from_le_bytes(self.buffer[self.inx..self.inx + SIZE].try_into().unwrap());
        self.inx += SIZE;
        value
    }

    #[inline]
    fn random_u64(&mut self) -> u64 {
        const SIZE: usize = core::mem::size_of::<u64>();
        if self.inx + SIZE > self.buffer.len() {
            self.generate_block();
            self.inx = 0;
        }
        let value = u64::from_le_bytes(self.buffer[self.inx..self.inx + SIZE].try_into().unwrap());
        self.inx += SIZE;
        value
    }

    #[inline]
    fn fill_u8(&mut self, destination: &mut [u8])
    where
        Self: Sized,
    {
        let mut out_inx: usize = 0;
        while out_inx < destination.len() {
            if self.inx == self.buffer.len() {
                self.generate_block();
                self.inx = 0;
            }
            let to_copy = usize::min(self.buffer.len() - self.inx, destination.len() - out_inx);
            debug_assert!(to_copy > 0);
            destination[out_inx..(out_inx + to_copy)]
                .copy_from_slice(&self.buffer[self.inx..(self.inx + to_copy)]);
            out_inx += to_copy;
            self.inx += to_copy;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tc8_chacha8() {
        // This is TC8 from https://github.com/secworks/chacha_testvectors/blob/master/src/chacha_testvectors.txt
        let mut rng = ChaCha::<8>::new(
            &[
                0xc4, 0x6e, 0xc1, 0xb1, 0x8c, 0xe8, 0xa8, 0x78, 0x72, 0x5a, 0x37, 0xe7, 0x80, 0xdf,
                0xb7, 0x35, 0x1f, 0x68, 0xed, 0x2e, 0x19, 0x4c, 0x79, 0xfb, 0xc6, 0xae, 0xbe, 0xe1,
                0xa6, 0x67, 0x97, 0x5d,
            ],
            [0x1a, 0xda, 0x31, 0xd5, 0xcf, 0x68, 0x82, 0x21],
        );

        let mut output = [0u8; 128];
        rng.fill_u8(&mut output);

        assert_eq!(
            output,
            [
                0x83, 0x87, 0x51, 0xb4, 0x2d, 0x8d, 0xdd, 0x8a, 0x3d, 0x77, 0xf4, 0x88, 0x25, 0xa2,
                0xba, 0x75, 0x2c, 0xf4, 0x04, 0x7c, 0xb3, 0x08, 0xa5, 0x97, 0x8e, 0xf2, 0x74, 0x97,
                0x3b, 0xe3, 0x74, 0xc9, 0x6a, 0xd8, 0x48, 0x06, 0x58, 0x71, 0x41, 0x7b, 0x08, 0xf0,
                0x34, 0xe6, 0x81, 0xfe, 0x46, 0xa9, 0x3f, 0x7d, 0x5c, 0x61, 0xd1, 0x30, 0x66, 0x14,
                0xd4, 0xaa, 0xf2, 0x57, 0xa7, 0xcf, 0xf0, 0x8b, 0x16, 0xf2, 0xfd, 0xa1, 0x70, 0xcc,
                0x18, 0xa4, 0xb5, 0x8a, 0x26, 0x67, 0xed, 0x96, 0x27, 0x74, 0xaf, 0x79, 0x2a, 0x6e,
                0x7f, 0x3c, 0x77, 0x99, 0x25, 0x40, 0x71, 0x1a, 0x7a, 0x13, 0x6d, 0x7e, 0x8a, 0x2f,
                0x8d, 0x3f, 0x93, 0x81, 0x67, 0x09, 0xd4, 0x5a, 0x3f, 0xa5, 0xf8, 0xce, 0x72, 0xfd,
                0xe1, 0x5b, 0xe7, 0xb8, 0x41, 0xac, 0xba, 0x3a, 0x2a, 0xbd, 0x55, 0x72, 0x28, 0xd9,
                0xfe, 0x4f,
            ]
        );
    }

    #[test]
    fn tc1_chacha12() {
        // This is TC1 from https://github.com/secworks/chacha_testvectors/blob/master/src/chacha_testvectors.txt
        let mut rng = ChaCha::<12>::new(&[0; 32], [0; 8]);

        let mut output = [0u8; 128];
        rng.fill_u8(&mut output);

        assert_eq!(
            output,
            [
                0x9b, 0xf4, 0x9a, 0x6a, 0x07, 0x55, 0xf9, 0x53, 0x81, 0x1f, 0xce, 0x12, 0x5f, 0x26,
                0x83, 0xd5, 0x04, 0x29, 0xc3, 0xbb, 0x49, 0xe0, 0x74, 0x14, 0x7e, 0x00, 0x89, 0xa5,
                0x2e, 0xae, 0x15, 0x5f, 0x05, 0x64, 0xf8, 0x79, 0xd2, 0x7a, 0xe3, 0xc0, 0x2c, 0xe8,
                0x28, 0x34, 0xac, 0xfa, 0x8c, 0x79, 0x3a, 0x62, 0x9f, 0x2c, 0xa0, 0xde, 0x69, 0x19,
                0x61, 0x0b, 0xe8, 0x2f, 0x41, 0x13, 0x26, 0xbe, 0x0b, 0xd5, 0x88, 0x41, 0x20, 0x3e,
                0x74, 0xfe, 0x86, 0xfc, 0x71, 0x33, 0x8c, 0xe0, 0x17, 0x3d, 0xc6, 0x28, 0xeb, 0xb7,
                0x19, 0xbd, 0xcb, 0xcc, 0x15, 0x15, 0x85, 0x21, 0x4c, 0xc0, 0x89, 0xb4, 0x42, 0x25,
                0x8d, 0xcd, 0xa1, 0x4c, 0xf1, 0x11, 0xc6, 0x02, 0xb8, 0x97, 0x1b, 0x8c, 0xc8, 0x43,
                0xe9, 0x1e, 0x46, 0xca, 0x90, 0x51, 0x51, 0xc0, 0x27, 0x44, 0xa6, 0xb0, 0x17, 0xe6,
                0x93, 0x16,
            ]
        );
    }

    #[test]
    fn tc2_chacha12() {
        // This is TC2 from https://github.com/secworks/chacha_testvectors/blob/master/src/chacha_testvectors.txt
        let mut rng = ChaCha::<12>::new(
            &[
                0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ],
            [0; 8],
        );

        let mut output = [0u8; 128];
        rng.fill_u8(&mut output);

        assert_eq!(
            output,
            [
                0x12, 0x05, 0x6e, 0x59, 0x5d, 0x56, 0xb0, 0xf6, 0xee, 0xf0, 0x90, 0xf0, 0xcd, 0x25,
                0xa2, 0x09, 0x49, 0x24, 0x8c, 0x27, 0x90, 0x52, 0x5d, 0x0f, 0x93, 0x02, 0x18, 0xff,
                0x0b, 0x4d, 0xdd, 0x10, 0xa6, 0x00, 0x22, 0x39, 0xd9, 0xa4, 0x54, 0xe2, 0x9e, 0x10,
                0x7a, 0x7d, 0x06, 0xfe, 0xfd, 0xfe, 0xf0, 0x21, 0x0f, 0xeb, 0xa0, 0x44, 0xf9, 0xf2,
                0x9b, 0x17, 0x72, 0xc9, 0x60, 0xdc, 0x29, 0xc0, 0x0c, 0x73, 0x66, 0xc5, 0xcb, 0xc6,
                0x04, 0x24, 0x0e, 0x66, 0x5e, 0xb0, 0x2a, 0x69, 0x37, 0x2a, 0x7a, 0xf9, 0x79, 0xb2,
                0x6f, 0xbb, 0x78, 0x09, 0x2a, 0xc7, 0xc4, 0xb8, 0x80, 0x29, 0xa7, 0xc8, 0x54, 0x51,
                0x3b, 0xc2, 0x17, 0xbb, 0xfc, 0x7d, 0x90, 0x43, 0x2e, 0x30, 0x8e, 0xba, 0x15, 0xaf,
                0xc6, 0x5a, 0xeb, 0x48, 0xef, 0x10, 0x0d, 0x56, 0x01, 0xe6, 0xaf, 0xba, 0x25, 0x71,
                0x17, 0xa9,
            ]
        );
    }

    #[test]
    fn tc3_chacha12() {
        // This is TC3 from https://github.com/secworks/chacha_testvectors/blob/master/src/chacha_testvectors.txt
        let mut rng = ChaCha::<12>::new(
            &[
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ],
            [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        );

        let mut output = [0u8; 128];
        rng.fill_u8(&mut output);

        assert_eq!(
            output,
            [
                0x64, 0xb8, 0xbd, 0xf8, 0x7b, 0x82, 0x8c, 0x4b, 0x6d, 0xba, 0xf7, 0xef, 0x69, 0x8d,
                0xe0, 0x3d, 0xf8, 0xb3, 0x3f, 0x63, 0x57, 0x14, 0x41, 0x8f, 0x98, 0x36, 0xad, 0xe5,
                0x9b, 0xe1, 0x29, 0x69, 0x46, 0xc9, 0x53, 0xa0, 0xf3, 0x8e, 0xcf, 0xfc, 0x9e, 0xcb,
                0x98, 0xe8, 0x1d, 0x5d, 0x99, 0xa5, 0xed, 0xfc, 0x8f, 0x9a, 0x0a, 0x45, 0xb9, 0xe4,
                0x1e, 0xf3, 0xb3, 0x1f, 0x02, 0x8f, 0x1d, 0x0f, 0x55, 0x9d, 0xb4, 0xa7, 0xf2, 0x22,
                0xc4, 0x42, 0xfe, 0x23, 0xb9, 0xa2, 0x59, 0x6a, 0x88, 0x28, 0x51, 0x22, 0xee, 0x4f,
                0x13, 0x63, 0x89, 0x6e, 0xa7, 0x7c, 0xa1, 0x50, 0x91, 0x2a, 0xc7, 0x23, 0xbf, 0xf0,
                0x4b, 0x02, 0x6a, 0x2f, 0x80, 0x7e, 0x03, 0xb2, 0x9c, 0x02, 0x07, 0x7d, 0x7b, 0x06,
                0xfc, 0x1a, 0xb9, 0x82, 0x7c, 0x13, 0xc8, 0x01, 0x3a, 0x6d, 0x83, 0xbd, 0x3b, 0x52,
                0xa2, 0x6f,
            ]
        );
    }

    #[test]
    fn tc4_chacha12() {
        // This is TC4 from https://github.com/secworks/chacha_testvectors/blob/master/src/chacha_testvectors.txt
        let mut rng = ChaCha::<12>::new(&[0xff; 32], [0xff; 8]);

        let mut output = [0u8; 128];
        rng.fill_u8(&mut output);

        assert_eq!(
            output,
            [
                0x04, 0xbf, 0x88, 0xda, 0xe8, 0xe4, 0x7a, 0x22, 0x8f, 0xa4, 0x7b, 0x7e, 0x63, 0x79,
                0x43, 0x4b, 0xa6, 0x64, 0xa7, 0xd2, 0x8f, 0x4d, 0xab, 0x84, 0xe5, 0xf8, 0xb4, 0x64,
                0xad, 0xd2, 0x0c, 0x3a, 0xca, 0xa6, 0x9c, 0x5a, 0xb2, 0x21, 0xa2, 0x3a, 0x57, 0xeb,
                0x5f, 0x34, 0x5c, 0x96, 0xf4, 0xd1, 0x32, 0x2d, 0x0a, 0x2f, 0xf7, 0xa9, 0xcd, 0x43,
                0x40, 0x1c, 0xd5, 0x36, 0x63, 0x9a, 0x61, 0x5a, 0x5c, 0x94, 0x29, 0xb5, 0x5c, 0xa3,
                0xc1, 0xb5, 0x53, 0x54, 0x55, 0x96, 0x69, 0xa1, 0x54, 0xac, 0xa4, 0x6c, 0xd7, 0x61,
                0xc4, 0x1a, 0xb8, 0xac, 0xe3, 0x85, 0x36, 0x3b, 0x95, 0x67, 0x5f, 0x06, 0x8e, 0x18,
                0xdb, 0x5a, 0x67, 0x3c, 0x11, 0x29, 0x1b, 0xd4, 0x18, 0x78, 0x92, 0xa9, 0xa3, 0xa3,
                0x35, 0x14, 0xf3, 0x71, 0x2b, 0x26, 0xc1, 0x30, 0x26, 0x10, 0x32, 0x98, 0xed, 0x76,
                0xbc, 0x9a,
            ]
        );
    }

    #[test]
    fn tc5_chacha12() {
        // This is TC5 from https://github.com/secworks/chacha_testvectors/blob/master/src/chacha_testvectors.txt
        let mut rng = ChaCha::<12>::new(&[0x55; 32], [0x55; 8]);

        let mut output = [0u8; 128];
        rng.fill_u8(&mut output);

        assert_eq!(
            output,
            [
                0xa6, 0x00, 0xf0, 0x77, 0x27, 0xff, 0x93, 0xf3, 0xda, 0x00, 0xdd, 0x74, 0xcc, 0x3e,
                0x8b, 0xfb, 0x5c, 0xa7, 0x30, 0x2f, 0x6a, 0x0a, 0x29, 0x44, 0x95, 0x3d, 0xe0, 0x04,
                0x50, 0xee, 0xcd, 0x40, 0xb8, 0x60, 0xf6, 0x60, 0x49, 0xf2, 0xea, 0xed, 0x63, 0xb2,
                0xef, 0x39, 0xcc, 0x31, 0x0d, 0x2c, 0x48, 0x8f, 0x5d, 0x9a, 0x24, 0x1b, 0x61, 0x5d,
                0xc0, 0xab, 0x70, 0xf9, 0x21, 0xb9, 0x1b, 0x95, 0x14, 0x0e, 0xff, 0x4a, 0xa4, 0x95,
                0xac, 0x61, 0x28, 0x9b, 0x6b, 0xc5, 0x7d, 0xe0, 0x72, 0x41, 0x9d, 0x09, 0xda, 0xa7,
                0xa7, 0x24, 0x39, 0x90, 0xda, 0xf3, 0x48, 0xa8, 0xf2, 0x83, 0x1e, 0x59, 0x7c, 0xf3,
                0x79, 0xb3, 0xb2, 0x84, 0xf0, 0x0b, 0xda, 0x27, 0xa4, 0xc6, 0x80, 0x85, 0x37, 0x4a,
                0x8a, 0x5c, 0x38, 0xde, 0xd6, 0x2d, 0x11, 0x41, 0xca, 0xe0, 0xbb, 0x83, 0x8d, 0xdc,
                0x22, 0x32,
            ]
        );
    }

    #[test]
    fn tc6_chacha12() {
        // This is TC6 from https://github.com/secworks/chacha_testvectors/blob/master/src/chacha_testvectors.txt
        let mut rng = ChaCha::<12>::new(&[0xaa; 32], [0xaa; 8]);

        let mut output = [0u8; 128];
        rng.fill_u8(&mut output);

        assert_eq!(
            output,
            [
                0x85, 0x65, 0x05, 0xb0, 0x1d, 0x3b, 0x47, 0xaa, 0xe0, 0x3d, 0x6a, 0x97, 0xaa, 0x0f,
                0x03, 0x3a, 0x9a, 0xdc, 0xc9, 0x43, 0x77, 0xba, 0xbd, 0x86, 0x08, 0x86, 0x4f, 0xb3,
                0xf6, 0x25, 0xb6, 0xe3, 0x14, 0xf0, 0x86, 0x15, 0x8f, 0x9f, 0x72, 0x5d, 0x81, 0x1e,
                0xeb, 0x95, 0x3b, 0x7f, 0x74, 0x70, 0x76, 0xe4, 0xc3, 0xf6, 0x39, 0xfa, 0x84, 0x1f,
                0xad, 0x6c, 0x9a, 0x70, 0x9e, 0x62, 0x13, 0x97, 0x6d, 0xd6, 0xee, 0x9b, 0x5e, 0x1e,
                0x2e, 0x67, 0x6b, 0x1c, 0x9e, 0x2b, 0x82, 0xc2, 0xe9, 0x6c, 0x16, 0x48, 0x43, 0x7b,
                0xff, 0x2f, 0x01, 0x26, 0xb7, 0x4e, 0x8c, 0xe0, 0xa9, 0xb0, 0x6d, 0x17, 0x20, 0xac,
                0x0b, 0x6f, 0x09, 0x08, 0x6f, 0x28, 0xbc, 0x20, 0x15, 0x87, 0xf0, 0x53, 0x5e, 0xd9,
                0x38, 0x52, 0x70, 0xd0, 0x8b, 0x4a, 0x93, 0x82, 0xf1, 0x8f, 0x82, 0xdb, 0xde, 0x18,
                0x21, 0x0e,
            ]
        );
    }

    #[test]
    fn tc7_chacha12() {
        // This is TC7 from https://github.com/secworks/chacha_testvectors/blob/master/src/chacha_testvectors.txt
        let mut rng = ChaCha::<12>::new(
            &[
                0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
                0xee, 0xff, 0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88, 0x77, 0x66, 0x55, 0x44,
                0x33, 0x22, 0x11, 0x00,
            ],
            [0x0f, 0x1e, 0x2d, 0x3c, 0x4b, 0x5a, 0x69, 0x78],
        );

        let mut output = [0u8; 128];
        rng.fill_u8(&mut output);

        assert_eq!(
            output,
            [
                0x7e, 0xd1, 0x2a, 0x3a, 0x63, 0x91, 0x2a, 0xe9, 0x41, 0xba, 0x6d, 0x4c, 0x0d, 0x5e,
                0x86, 0x2e, 0x56, 0x8b, 0x0e, 0x55, 0x89, 0x34, 0x69, 0x35, 0x50, 0x5f, 0x06, 0x4b,
                0x8c, 0x26, 0x98, 0xdb, 0xf7, 0xd8, 0x50, 0x66, 0x7d, 0x8e, 0x67, 0xbe, 0x63, 0x9f,
                0x3b, 0x4f, 0x6a, 0x16, 0xf9, 0x2e, 0x65, 0xea, 0x80, 0xf6, 0xc7, 0x42, 0x94, 0x45,
                0xda, 0x1f, 0xc2, 0xc1, 0xb9, 0x36, 0x50, 0x40, 0xe3, 0x2e, 0x50, 0xc4, 0x10, 0x6f,
                0x3b, 0x3d, 0xa1, 0xce, 0x7c, 0xcb, 0x1e, 0x71, 0x40, 0xb1, 0x53, 0x49, 0x3c, 0x0f,
                0x3a, 0xd9, 0xa9, 0xbc, 0xff, 0x07, 0x7e, 0xc4, 0x59, 0x6f, 0x1d, 0x0f, 0x29, 0xbf,
                0x9c, 0xba, 0xa5, 0x02, 0x82, 0x0f, 0x73, 0x2a, 0xf5, 0xa9, 0x3c, 0x49, 0xee, 0xe3,
                0x3d, 0x1c, 0x4f, 0x12, 0xaf, 0x3b, 0x42, 0x97, 0xaf, 0x91, 0xfe, 0x41, 0xea, 0x9e,
                0x94, 0xa2,
            ]
        );
    }

    #[test]
    fn tc8_chacha12() {
        // This is TC8 from https://github.com/secworks/chacha_testvectors/blob/master/src/chacha_testvectors.txt
        let mut rng = ChaCha::<12>::new(
            &[
                0xc4, 0x6e, 0xc1, 0xb1, 0x8c, 0xe8, 0xa8, 0x78, 0x72, 0x5a, 0x37, 0xe7, 0x80, 0xdf,
                0xb7, 0x35, 0x1f, 0x68, 0xed, 0x2e, 0x19, 0x4c, 0x79, 0xfb, 0xc6, 0xae, 0xbe, 0xe1,
                0xa6, 0x67, 0x97, 0x5d,
            ],
            [0x1a, 0xda, 0x31, 0xd5, 0xcf, 0x68, 0x82, 0x21],
        );

        let mut output = [0u8; 128];
        rng.fill_u8(&mut output);

        assert_eq!(
            output,
            [
                0x14, 0x82, 0x07, 0x27, 0x84, 0xbc, 0x6d, 0x06, 0xb4, 0xe7, 0x3b, 0xdc, 0x11, 0x8b,
                0xc0, 0x10, 0x3c, 0x79, 0x76, 0x78, 0x6c, 0xa9, 0x18, 0xe0, 0x69, 0x86, 0xaa, 0x25,
                0x1f, 0x7e, 0x9c, 0xc1, 0xb2, 0x74, 0x9a, 0x0a, 0x16, 0xee, 0x83, 0xb4, 0x24, 0x2d,
                0x2e, 0x99, 0xb0, 0x8d, 0x7c, 0x20, 0x09, 0x2b, 0x80, 0xbc, 0x46, 0x6c, 0x87, 0x28,
                0x3b, 0x61, 0xb1, 0xb3, 0x9d, 0x0f, 0xfb, 0xab, 0xd9, 0x4b, 0x11, 0x6b, 0xc1, 0xeb,
                0xdb, 0x32, 0x9b, 0x9e, 0x4f, 0x62, 0x0d, 0xb6, 0x95, 0x54, 0x4a, 0x8e, 0x3d, 0x9b,
                0x68, 0x47, 0x3d, 0x0c, 0x97, 0x5a, 0x46, 0xad, 0x96, 0x6e, 0xd6, 0x31, 0xe4, 0x2a,
                0xff, 0x53, 0x0a, 0xd5, 0xea, 0xc7, 0xd8, 0x04, 0x7a, 0xdf, 0xa1, 0xe5, 0x11, 0x3c,
                0x91, 0xf3, 0xe3, 0xb8, 0x83, 0xf1, 0xd1, 0x89, 0xac, 0x1c, 0x8f, 0xe0, 0x7b, 0xa5,
                0xa4, 0x2b,
            ]
        );
    }

    #[test]
    fn tc8_chacha20() {
        // This is TC8 from https://github.com/secworks/chacha_testvectors/blob/master/src/chacha_testvectors.txt
        let mut rng = ChaCha::<20>::new(
            &[
                0xc4, 0x6e, 0xc1, 0xb1, 0x8c, 0xe8, 0xa8, 0x78, 0x72, 0x5a, 0x37, 0xe7, 0x80, 0xdf,
                0xb7, 0x35, 0x1f, 0x68, 0xed, 0x2e, 0x19, 0x4c, 0x79, 0xfb, 0xc6, 0xae, 0xbe, 0xe1,
                0xa6, 0x67, 0x97, 0x5d,
            ],
            [0x1a, 0xda, 0x31, 0xd5, 0xcf, 0x68, 0x82, 0x21],
        );

        let mut output = [0u8; 128];
        rng.fill_u8(&mut output);

        assert_eq!(
            output,
            [
                0xf6, 0x3a, 0x89, 0xb7, 0x5c, 0x22, 0x71, 0xf9, 0x36, 0x88, 0x16, 0x54, 0x2b, 0xa5,
                0x2f, 0x06, 0xed, 0x49, 0x24, 0x17, 0x92, 0x30, 0x2b, 0x00, 0xb5, 0xe8, 0xf8, 0x0a,
                0xe9, 0xa4, 0x73, 0xaf, 0xc2, 0x5b, 0x21, 0x8f, 0x51, 0x9a, 0xf0, 0xfd, 0xd4, 0x06,
                0x36, 0x2e, 0x8d, 0x69, 0xde, 0x7f, 0x54, 0xc6, 0x04, 0xa6, 0xe0, 0x0f, 0x35, 0x3f,
                0x11, 0x0f, 0x77, 0x1b, 0xdc, 0xa8, 0xab, 0x92, 0xe5, 0xfb, 0xc3, 0x4e, 0x60, 0xa1,
                0xd9, 0xa9, 0xdb, 0x17, 0x34, 0x5b, 0x0a, 0x40, 0x27, 0x36, 0x85, 0x3b, 0xf9, 0x10,
                0xb0, 0x60, 0xbd, 0xf1, 0xf8, 0x97, 0xb6, 0x29, 0x0f, 0x01, 0xd1, 0x38, 0xae, 0x2c,
                0x4c, 0x90, 0x22, 0x5b, 0xa9, 0xea, 0x14, 0xd5, 0x18, 0xf5, 0x59, 0x29, 0xde, 0xa0,
                0x98, 0xca, 0x7a, 0x6c, 0xcf, 0xe6, 0x12, 0x27, 0x05, 0x3c, 0x84, 0xe4, 0x9a, 0x4a,
                0x33, 0x32,
            ]
        );
    }

    #[test]
    fn chacha8_byte_output() {
        // This test uses the same test vector as the Chacha crate, to verify against
        // another independent implementation:
        let mut rng = ChaCha::<8>::new(
            &[
                0x64, 0x1a, 0xea, 0xeb, 0x08, 0x03, 0x6b, 0x61, 0x7a, 0x42, 0xcf, 0x14, 0xe8, 0xc5,
                0xd2, 0xd1, 0x15, 0xf8, 0xd7, 0xcb, 0x6e, 0xa5, 0xe2, 0x8b, 0x9b, 0xfa, 0xf8, 0x3e,
                0x03, 0x84, 0x26, 0xa7,
            ],
            [0xa1, 0x4a, 0x11, 0x68, 0x27, 0x1d, 0x45, 0x9b],
        );

        // Fill the output in stages, to test the edge cases of the fill_u8 function:
        let mut output = [0u8; 96];
        rng.fill_u8(&mut output[0..8]);
        rng.fill_u8(&mut output[8..72]);
        rng.fill_u8(&mut output[72..96]);

        assert_eq!(
            output,
            [
                0x17, 0x21, 0xc0, 0x44, 0xa8, 0xa6, 0x45, 0x35, 0x22, 0xdd, 0xdb, 0x31, 0x43, 0xd0,
                0xbe, 0x35, 0x12, 0x63, 0x3c, 0xa3, 0xc7, 0x9b, 0xf8, 0xcc, 0xc3, 0x59, 0x4c, 0xb2,
                0xc2, 0xf3, 0x10, 0xf7, 0xbd, 0x54, 0x4f, 0x55, 0xce, 0x0d, 0xb3, 0x81, 0x23, 0x41,
                0x2d, 0x6c, 0x45, 0x20, 0x7d, 0x5c, 0xf9, 0xaf, 0x0c, 0x6c, 0x68, 0x0c, 0xce, 0x1f,
                0x7e, 0x43, 0x38, 0x8d, 0x1b, 0x03, 0x46, 0xb7, 0x13, 0x3c, 0x59, 0xfd, 0x6a, 0xf4,
                0xa5, 0xa5, 0x68, 0xaa, 0x33, 0x4c, 0xcd, 0xc3, 0x8a, 0xf5, 0xac, 0xe2, 0x01, 0xdf,
                0x84, 0xd0, 0xa3, 0xca, 0x22, 0x54, 0x94, 0xca, 0x62, 0x09, 0x34, 0x5f,
            ]
        );
    }

    #[test]
    fn chacha12_byte_output() {
        // Test vector from the Chacha crate, to verify against
        // another independent implementation:
        let mut rng = ChaCha::<12>::new(
            &[
                0x27, 0xfc, 0x12, 0x0b, 0x01, 0x3b, 0x82, 0x9f, 0x1f, 0xae, 0xef, 0xd1, 0xab, 0x41,
                0x7e, 0x86, 0x62, 0xf4, 0x3e, 0x0d, 0x73, 0xf9, 0x8d, 0xe8, 0x66, 0xe3, 0x46, 0x35,
                0x31, 0x80, 0xfd, 0xb7,
            ],
            [0xdb, 0x4b, 0x4a, 0x41, 0xd8, 0xdf, 0x18, 0xaa],
        );

        let mut output = [0u8; 96];
        rng.fill_u8(&mut output);

        assert_eq!(
            output,
            [
                0x5f, 0x3c, 0x8c, 0x19, 0x0a, 0x78, 0xab, 0x7f, 0xe8, 0x08, 0xca, 0xe9, 0xcb, 0xcb,
                0x0a, 0x98, 0x37, 0xc8, 0x93, 0x49, 0x2d, 0x96, 0x3a, 0x1c, 0x2e, 0xda, 0x6c, 0x15,
                0x58, 0xb0, 0x2c, 0x83, 0xfc, 0x02, 0xa4, 0x4c, 0xbb, 0xb7, 0xe6, 0x20, 0x4d, 0x51,
                0xd1, 0xc2, 0x43, 0x0e, 0x9c, 0x0b, 0x58, 0xf2, 0x93, 0x7b, 0xf5, 0x93, 0x84, 0x0c,
                0x85, 0x0b, 0xda, 0x90, 0x51, 0xa1, 0xf0, 0x51, 0xdd, 0xf0, 0x9d, 0x2a, 0x03, 0xeb,
                0xf0, 0x9f, 0x01, 0xbd, 0xba, 0x9d, 0xa0, 0xb6, 0xda, 0x79, 0x1b, 0x2e, 0x64, 0x56,
                0x41, 0x04, 0x7d, 0x11, 0xeb, 0xf8, 0x50, 0x87, 0xd4, 0xde, 0x5c, 0x01,
            ]
        );
    }

    #[test]
    fn chacha12_u32_output() {
        let mut rng = ChaCha::<12>::new(
            &[
                0x27, 0xfc, 0x12, 0x0b, 0x01, 0x3b, 0x82, 0x9f, 0x1f, 0xae, 0xef, 0xd1, 0xab, 0x41,
                0x7e, 0x86, 0x62, 0xf4, 0x3e, 0x0d, 0x73, 0xf9, 0x8d, 0xe8, 0x66, 0xe3, 0x46, 0x35,
                0x31, 0x80, 0xfd, 0xb7,
            ],
            [0xdb, 0x4b, 0x4a, 0x41, 0xd8, 0xdf, 0x18, 0xaa],
        );

        let output: [u32; 24] = core::array::from_fn(|_| rng.random_u32());

        assert_eq!(
            output,
            [
                0x198c3c5f, 0x7fab780a, 0xe9ca08e8, 0x980acbcb, 0x4993c837, 0x1c3a962d, 0x156cda2e,
                0x832cb058, 0x4ca402fc, 0x20e6b7bb, 0xc2d1514d, 0x0b9c0e43, 0x7b93f258, 0x0c8493f5,
                0x90da0b85, 0x51f0a151, 0x2a9df0dd, 0x9ff0eb03, 0x9dbabd01, 0x79dab6a0, 0x56642e1b,
                0x117d0441, 0x8750f8eb, 0x015cded4,
            ]
        );
    }

    #[test]
    fn chacha12_u64_output() {
        let mut rng = ChaCha::<12>::new(
            &[
                0x27, 0xfc, 0x12, 0x0b, 0x01, 0x3b, 0x82, 0x9f, 0x1f, 0xae, 0xef, 0xd1, 0xab, 0x41,
                0x7e, 0x86, 0x62, 0xf4, 0x3e, 0x0d, 0x73, 0xf9, 0x8d, 0xe8, 0x66, 0xe3, 0x46, 0x35,
                0x31, 0x80, 0xfd, 0xb7,
            ],
            [0xdb, 0x4b, 0x4a, 0x41, 0xd8, 0xdf, 0x18, 0xaa],
        );

        let output: [u64; 12] = core::array::from_fn(|_| rng.random_u64());

        assert_eq!(
            output,
            [
                0x7fab780a198c3c5f,
                0x980acbcbe9ca08e8,
                0x1c3a962d4993c837,
                0x832cb058156cda2e,
                0x20e6b7bb4ca402fc,
                0x0b9c0e43c2d1514d,
                0x0c8493f57b93f258,
                0x51f0a15190da0b85,
                0x9ff0eb032a9df0dd,
                0x79dab6a09dbabd01,
                0x117d044156642e1b,
                0x015cded48750f8eb,
            ]
        );
    }
}
