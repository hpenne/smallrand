#![forbid(unsafe_code)]
#![allow(clippy::inline_always)]

#[cfg(all(unix, feature = "std"))]
use crate::devices::DevUrandom;
#[cfg(all(not(unix), feature = "std"))]
use crate::GetRandom;
use crate::{RandomDevice, Rng};
use std::ops::BitXor;
#[cfg(feature = "std")]
use std::time::SystemTime;

#[allow(clippy::module_name_repetitions)]
pub struct ChaCha8(ChaCha<8>);

impl ChaCha8 {
    #[cfg(feature = "std")]
    #[must_use]
    pub fn new() -> Self {
        let duration_since_epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        #[allow(clippy::cast_possible_truncation)]
        let nonce = (duration_since_epoch.as_nanos() as u64).to_ne_bytes();
        #[cfg(unix)]
        let rng = Self::from_device(&mut DevUrandom::new(), nonce);
        #[cfg(not(unix))]
        let rng = Self::from_device(&mut GetRandom::new(), nonce);
        rng
    }

    pub fn from_device<T>(random_device: &mut T, nonce: [u8; 8]) -> Self
    where
        T: RandomDevice,
    {
        let seed = random_device.seed_bytes();
        Self(ChaCha::<8>::new(&seed, nonce))
    }
}

impl Rng for ChaCha8 {
    #[inline(always)]
    fn random_u32(&mut self) -> u32 {
        self.0.random_u32()
    }

    #[inline(always)]
    fn random_u64(&mut self) -> u64 {
        self.0.random_u64()
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

    #[inline]
    fn generate_block(&mut self) {
        let mut x = [0_u32; 16];
        x.copy_from_slice(&self.state);
        for _round in (0..ROUNDS).step_by(2) {
            Self::quarter_round(&mut x, 0, 4, 8, 12);
            Self::quarter_round(&mut x, 1, 5, 9, 13);
            Self::quarter_round(&mut x, 2, 6, 10, 14);
            Self::quarter_round(&mut x, 3, 7, 11, 15);
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
            self.state[13] = self.state[13].wrapping_add(1);
            assert_ne!(0, self.state[13], "Max number of bytes exceeded");
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
    fn random_u32(&mut self) -> u32 {
        if self.inx + core::mem::size_of::<u32>() >= self.buffer.len() {
            self.generate_block();
            self.inx = 0;
        }
        u32::from_le_bytes(self.buffer[self.inx..self.inx + 4].try_into().unwrap())
    }

    fn random_u64(&mut self) -> u64 {
        if self.inx + core::mem::size_of::<u64>() >= self.buffer.len() {
            self.generate_block();
            self.inx = 0;
        }
        u64::from_le_bytes(self.buffer[self.inx..self.inx + 8].try_into().unwrap())
    }

    fn fill_u8(&mut self, destination: &mut [u8])
    where
        Self: Sized,
    {
        for element in destination.iter_mut() {
            if self.inx == self.buffer.len() {
                self.generate_block();
                self.inx = 0;
            }
            *element = self.buffer[self.inx];
            self.inx += 1;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn chacha8_case_1() {
        let mut stream = ChaCha::<8>::new(
            &[
                0x64, 0x1a, 0xea, 0xeb, 0x08, 0x03, 0x6b, 0x61, 0x7a, 0x42, 0xcf, 0x14, 0xe8, 0xc5,
                0xd2, 0xd1, 0x15, 0xf8, 0xd7, 0xcb, 0x6e, 0xa5, 0xe2, 0x8b, 0x9b, 0xfa, 0xf8, 0x3e,
                0x03, 0x84, 0x26, 0xa7,
            ],
            [0xa1, 0x4a, 0x11, 0x68, 0x27, 0x1d, 0x45, 0x9b],
        );

        let mut xs = [0u8; 100];
        stream.fill_u8(&mut xs);

        assert_eq!(
            xs.to_vec(),
            [
                0x17, 0x21, 0xc0, 0x44, 0xa8, 0xa6, 0x45, 0x35, 0x22, 0xdd, 0xdb, 0x31, 0x43, 0xd0,
                0xbe, 0x35, 0x12, 0x63, 0x3c, 0xa3, 0xc7, 0x9b, 0xf8, 0xcc, 0xc3, 0x59, 0x4c, 0xb2,
                0xc2, 0xf3, 0x10, 0xf7, 0xbd, 0x54, 0x4f, 0x55, 0xce, 0x0d, 0xb3, 0x81, 0x23, 0x41,
                0x2d, 0x6c, 0x45, 0x20, 0x7d, 0x5c, 0xf9, 0xaf, 0x0c, 0x6c, 0x68, 0x0c, 0xce, 0x1f,
                0x7e, 0x43, 0x38, 0x8d, 0x1b, 0x03, 0x46, 0xb7, 0x13, 0x3c, 0x59, 0xfd, 0x6a, 0xf4,
                0xa5, 0xa5, 0x68, 0xaa, 0x33, 0x4c, 0xcd, 0xc3, 0x8a, 0xf5, 0xac, 0xe2, 0x01, 0xdf,
                0x84, 0xd0, 0xa3, 0xca, 0x22, 0x54, 0x94, 0xca, 0x62, 0x09, 0x34, 0x5f, 0xcf, 0x30,
                0x13, 0x2e,
            ]
            .to_vec()
        );
    }
}
