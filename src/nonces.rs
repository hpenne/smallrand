//! This module implements a function to generate nonces for
//! use in algorithm that need this.

use core::sync::atomic::{AtomicU64, Ordering};
#[cfg(feature = "std")]
use std::collections::hash_map::RandomState;
#[cfg(feature = "std")]
use std::hash::{BuildHasher, Hash, Hasher};

// Define a global static atomic counter
static NONCE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// This is intended as an independent random source for the ChaCha nonce.
/// It needs to use alternative (and thus potentially less secure) sources
/// of entropy in order to still provide some randomness if the standard
/// source breaks.
/// It is NOT cryptographically secure.
#[cfg(feature = "std")]
pub fn nonce_u64() -> [u8; 8] {
    // Get the time, and XOR seconds with nanoseconds:
    let duration_since_epoch = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap();
    let nanos = duration_since_epoch.as_nanos();
    #[allow(clippy::cast_possible_truncation)]
    let from_time = ((nanos >> 64) as u64) ^ (nanos as u64) ^ duration_since_epoch.as_secs();

    // Increment and get the global counter:
    let from_counter = NONCE_COUNTER.fetch_add(1, Ordering::Relaxed);

    // Pass these two values through a hashed built from RandomState, which is in itself
    // a source of entropy:
    let mut hasher = RandomState::new().build_hasher();
    from_time.hash(&mut hasher);
    from_counter.hash(&mut hasher);

    // Finally we do an XOR with time and count,
    // just in case the DefaultHasher is broken:
    (hasher.finish() ^ from_time ^ from_counter).to_ne_bytes()
}

#[cfg(not(feature = "std"))]
pub fn nonce_u64() -> [u8; 8] {
    // We have no time and no hasher, so all we can do
    // is to increment and get the global counter:
    NONCE_COUNTER.fetch_add(1, Ordering::Relaxed).to_ne_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonces_are_different() {
        assert_ne!(nonce_u64(), nonce_u64());
    }
}
