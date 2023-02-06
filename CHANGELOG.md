# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Add `wait` method variants to windivert
- Add `WinDivert::<()>::MAX_BATCH`

### Changed

- Handle `MAX_BATCH` limit in `send_ex`

## [Unreleased-sys]

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

[unreleased]: https://github.com/Rubensei/windivert-rust/compare/windivert-0.5.4...HEAD
[unreleased-sys]: https://github.com/Rubensei/windivert-rust/compare/windivert-sys-0.9.1...HEAD
[sys-0.9.1]: https://github.com/Rubensei/windivert-rust/compare/windivert-sys-0.9.0...windivert-sys-0.9.1
[0.5.4]: https://github.com/Rubensei/windivert-rust/compare/windivert-0.5.3...windivert-0.5.4
[0.5.3]: https://github.com/Rubensei/windivert-rust/compare/windivert-0.5.1...windivert-0.5.3
[sys-0.9.0]: https://github.com/Rubensei/windivert-rust/releases/tag/windivert-sys-0.9.0
[0.5.1]: https://github.com/Rubensei/windivert-rust/compare/windivert-0.5.0...windivert-0.5.1
[0.5.0]: https://github.com/Rubensei/windivert-rust/releases/tag/windivert-0.5.0
