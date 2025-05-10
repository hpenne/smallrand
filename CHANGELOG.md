# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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