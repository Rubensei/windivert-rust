# WinDivert 2 Rust Wrapper

This projects allows you to use [WinDivert](https://www.reqrypt.org/windivert.html) from rust. It consist of two crates:
 * `windivert-sys`: Crate providing raw bindings to the WinDivert user mode library.
 * `windivert`: (WIP) Built on top of `windivert-sys` and providing a friendlier Rust API and some abstractions.

# Build
To be able to build `windivert-sys` (or `windivert`, since it dependes on it) you require the WinDivert library files to be able to link against it.

The path for the files can be specified using `WINDIVERT_LIB` environment variable

**WIP: Compile windivert library from source as fallback if the env variable is not set**

# Usage
 * `windivert-sys` shares the same API the native library uses. Read [official documentation](https://www.reqrypt.org/windivert-doc.html) for more details.
 * `windivert` WIP