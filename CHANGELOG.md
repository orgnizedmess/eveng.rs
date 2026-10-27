# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.1.2] - 2026-10-28

- node/network: Added a getter to access node and network IDs from `NodeClient` and `NetworkClient` respectively ([#21])
- crate: Added two Cargo features---`rustls` and `native-tls`---to let consumers choose the TLS backend ([#24])
- client: Fixed a bug in the request handling, whn error case for a success body that does not parse ([#22])
- docs: Added a note to clarify how `disconnect` works ([#25])

## [0.1.1] - 2026-09-22

- docs: Added badges to README for CI, docs.rs and crates.io
- client: Removed print statements originally added for logging

## [0.1.0] - 2026-09-20

Initial release

[Unreleased]: https://github.com/orgnizedmess/eveng.rs/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/orgnizedmess/eveng.rs/releases/tag/v0.1.2
[0.1.1]: https://github.com/orgnizedmess/eveng.rs/releases/tag/v0.1.1
[0.1.0]: https://github.com/orgnizedmess/eveng.rs/releases/tag/v0.1.0

[#25]: https://github.com/orgnizedmess/eveng.rs/pull/25
[#24]: https://github.com/orgnizedmess/eveng.rs/pull/24
[#22]: https://github.com/orgnizedmess/eveng.rs/pull/22
[#21]: https://github.com/orgnizedmess/eveng.rs/pull/21
