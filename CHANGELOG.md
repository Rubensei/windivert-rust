# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Add `wait` methods back
- Partial single recv `WinDivert::partial_recv()`
- `WinDivertSendError`
- Add `WinDivert<()>::install(path: &Path)`

### Changed

- `WinDivertError` has a `Send` variant
- `WinDivert::recv()` and `WinDivert::recv_ex()` buffer made mandatory on data
  capturing layers and removed from non-capturing layers.
- Bumped `windows-rs` to 0.51.1
- Code refactor and cleanup
- Removed IOError variant from `WinDivertError`

### Fixed

- All generic OS errors will be properly handled as `WinDivertError::OsError`

## [Unreleased-sys]

### Changed

- Remove `windows` to decouple this crate from `windows-sys`
- Replace `std::os::raw` with `core::ffi`
- Fix typo in enum variant name: `WinDivertEvent::FlowStablished` to
  `WinDivertEvent::FlowEstablished`

## [0.6.0]

### Added

- Add `static` feature to statically link to windivert library.

## [sys-0.10.0]

### Added

- Add `WinDivertFlags::<layer>_default` methods.
- Add `static` feature to statically link to windivert library.

### Changed

- Refactor build scripts.

## [sys-0.9.3] - 2023-04-03

### Fixed

- Fix wrong comparison in `MF` and `DF` flag getters

## [sys-0.9.2] - 2023-03-23

### Fixed

- Downgrade windows to `0.43` to avoid build issues with `windivert` due to
  different windows versions

## [0.5.5] - 2023-03-23

### Added

- Add `WinDivert::<()>::MAX_BATCH`

### Fixed

- Fix `close` errors due to double inner close call

## [sys-0.9.1] - 2023-02-06

### Fixed

- Fix logic error in bitflag methods
  ([#2](https://github.com/Rubensei/windivert-rust/issues/2))
- Fix incorrect links in documentation
  ([#3](https://github.com/Rubensei/windivert-rust/issues/3))

## [0.5.3] - 2023-01-07

### Fixed

- Fix `send_ex` error due to incorrect use of iterator.

## [0.5.3] - 2023-01-03

### Changed

- Add Debug and Clone trait bounds to the types used for typestate pattern

## [sys-0.9.0] - 2022-12-21

### Added

- Initial tagged release

## [0.5.1] - 2022-12-21

### Changed

- Make typestate types public

## [0.5.0] - 2022-12-21

### Added

- Initial tagged release

[unreleased]: https://github.com/Rubensei/windivert-rust/compare/windivert-0.6.0...HEAD
[unreleased-sys]: https://github.com/Rubensei/windivert-rust/compare/windivert-sys-0.10.0...HEAD
[0.6.0]: https://github.com/Rubensei/windivert-rust/compare/windivert-0.5.5...windivert-0.6.0
[sys-0.10.0]: https://github.com/Rubensei/windivert-rust/compare/windivert-sys-0.9.3...windivert-sys-0.10.0
[sys-0.9.3]: https://github.com/Rubensei/windivert-rust/compare/windivert-sys-0.9.2...windivert-sys-0.9.3
[sys-0.9.2]: https://github.com/Rubensei/windivert-rust/compare/windivert-sys-0.9.1...windivert-sys-0.9.2
[0.5.5]: https://github.com/Rubensei/windivert-rust/compare/windivert-0.5.4...windivert-0.5.5
[sys-0.9.1]: https://github.com/Rubensei/windivert-rust/compare/windivert-sys-0.9.0...windivert-sys-0.9.1
[0.5.4]: https://github.com/Rubensei/windivert-rust/compare/windivert-0.5.3...windivert-0.5.4
[0.5.3]: https://github.com/Rubensei/windivert-rust/compare/windivert-0.5.1...windivert-0.5.3
[sys-0.9.0]: https://github.com/Rubensei/windivert-rust/releases/tag/windivert-sys-0.9.0
[0.5.1]: https://github.com/Rubensei/windivert-rust/compare/windivert-0.5.0...windivert-0.5.1
[0.5.0]: https://github.com/Rubensei/windivert-rust/releases/tag/windivert-0.5.0
