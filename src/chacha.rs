#![forbid(unsafe_code)]

#[cfg(all(unix, feature = "std"))]
use crate::devices::DevUrandom;
#[cfg(all(not(unix), feature = "std"))]
use crate::GetRandom;
use crate::{RandomDevice, Rng};
use chacha::KeyStream;
#[cfg(feature = "std")]
use std::time::SystemTime;

#[allow(clippy::module_name_repetitions)]
pub struct ChaCha8(chacha::ChaCha);

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
        Self(chacha::ChaCha::new_chacha8(&seed, &nonce))
    }
}

impl Rng for ChaCha8 {
    fn random_u32(&mut self) -> u32 {
        let mut bytes = [0_u8; 4];
        self.0.xor_read(&mut bytes).expect("ChaCha failed");
        u32::from_be_bytes(bytes)
    }

    fn random_u64(&mut self) -> u64 {
        let mut bytes = [0_u8; 8];
        self.0.xor_read(&mut bytes).expect("ChaCha failed");
        u64::from_be_bytes(bytes)
    }
}
