//! This module implements a function to generate nonces for
//! use in algorithm that need this.

#[cfg(feature = "std")]
use core::ops::BitXor;
use core::sync::atomic::{AtomicU64, Ordering};
#[cfg(feature = "std")]
use std::hash::{BuildHasher, RandomState};
#[cfg(feature = "std")]
use std::hash::{Hash, Hasher};

// Define a global static atomic counter
static NONCE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[cfg(feature = "std")]
pub fn nonce_u64() -> [u8; 8] {
    // Get the time, and XOR seconds with nanoseconds:
    let duration_since_epoch = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap();
    #[allow(clippy::cast_possible_truncation)]
    let from_time = (duration_since_epoch.as_nanos() as u64).bitxor(duration_since_epoch.as_secs());

    // Increment and get the global counter:
    let from_counter = NONCE_COUNTER.fetch_add(1, Ordering::SeqCst);

    // Pass these two values through a hashed built from RandomState, which is in itself
    // a source of entropy:
    let mut hasher = RandomState::new().build_hasher();
    from_time.hash(&mut hasher);
    from_counter.hash(&mut hasher);

    // Finally we do an XOR with time and count,
    // just in case the DefaultHasher is broken:
    hasher
        .finish()
        .bitxor(from_time)
        .bitxor(from_counter)
        .to_ne_bytes()
}

#[cfg(not(feature = "std"))]
pub fn nonce_u64() -> [u8; 8] {
    // We have no time and no hasher, so all we can do
    // is to increment and get the global counter:
    NONCE_COUNTER.fetch_add(1, Ordering::SeqCst).to_ne_bytes()
}
