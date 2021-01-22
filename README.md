# WinDivert 2 Rust Wrapper
[![GitHub](https://img.shields.io/github/license/Rubensei/windivert-rust?color=blue)](https://raw.githubusercontent.com/rust-lang/docs.rs/master/LICENSE)

**Note**: This is a work in project, so the crates won't follow semantic versioning until 1.0.0 release, so any version change below 1.0.0 might introduce breaking changes in the API or the crate usage in general.

This projects allows you to use [WinDivert](https://www.reqrypt.org/windivert.html) from rust. It consist of two crates:
 * `windivert-sys` [![crates.io](https://img.shields.io/crates/v/windivert-sys)](https://crates.io/crates/windivert-sys) [![docs](https://docs.rs/windivert-sys/badge.svg)](https://docs.rs/windivert-sys/): Crate providing raw bindings to the WinDivert user mode library.
 * `windivert` [![crates.io](https://img.shields.io/crates/v/windivert)](https://crates.io/crates/windivert) [![docs](https://docs.rs/windivert/badge.svg)](https://docs.rs/windivert/): (WIP) Built on top of `windivert-sys` and providing a friendlier Rust API and some abstractions.

# Build
To be able to build `windivert-sys` (or `windivert`, since it dependes on it) you require the WinDivert library files to be able to link against it.

The path for the files can be specified using `WINDIVERT_LIB` environment variable

**WIP: Compile windivert library from source as fallback if the env variable is not set**

# Usage
 * `windivert-sys` shares the same API the native library uses. Read [official documentation](https://www.reqrypt.org/windivert-doc.html) for more details.
 * `windivert` WIP