# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.1](https://github.com/hpenne/smallrand/compare/v1.0.0...v1.0.1) - 2025-06-25

### Fixed

- Replace SecureEntropy with DefaultEntropy for SmallRng::default ([#36](https://github.com/hpenne/smallrand/pull/36))
- RepetitionCountTester threshold off-by-one error that caused a higher false alarm rate than desired ([#37](https://github.com/hpenne/smallrand/pull/37))

## [1.0.0](https://github.com/hpenne/smallrand/compare/v0.2.1...v1.0.0) - 2025-06-04

### Other

- Removed iter u8
- More test vectors for ChaCha12, etc.
- Updated readme
- Fixed a bug in generation of f32 in a range that caused very small values to be 0 (if the 41 most significant bits of the raw u64 value were all 0).
- Fixed the construction of the hasher in nonce_u64 to properly mix in RandomState entropy in the nonce (inspecting the source for DefaultHasher::new revealed that it seemed to use constant keys that are 0).

## [0.2.1](https://github.com/hpenne/smallrand/compare/v0.2.0...v0.2.1) - 2025-05-16

### Added

- HashMapEntropy ([#30](https://github.com/hpenne/smallrand/pull/30))
- On-by-default feature flag "allow-getrandom" that can be turned off to use HashMapEntropy instead of getrandom on
  non-Unix-like platform and remove the dependency on getrandom.

## [0.2.0](https://github.com/hpenne/smallrand/compare/v0.1.0...v0.2.0) - 2025-05-10

### Added

- Added SecureEntropy
- Added DefaultEntropy
- Added SplitMix
- SmallRng::from_seed

### Other

- Renamed `RandomDevice` to `EntropySource`.
- Tweaks and doc. ([#27](https://github.com/hpenne/smallrand/pull/27))
- Test vectors ([#24](https://github.com/hpenne/smallrand/pull/24))
- Updated readme.md
- Modified Rng::fill_u8 to use to_le_bytes, to ensure the same output on all platforms.
- Improved nonce generation by adding the DefaultHasher and a global counter as additional
  sources. ([#22](https://github.com/hpenne/smallrand/pull/22))

## [0.1.0]

Initial version