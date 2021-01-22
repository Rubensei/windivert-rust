//! Raw FFI bindings for [WinDivert].
//!
//! For more information, refer to [WinDivert's documentation].
//!
//! [WinDivert]: https://www.reqrypt.org/windivert.html
//! [WinDivert's documentation]: https://www.reqrypt.org/windivert-doc.html

extern crate winapi;

#[allow(dead_code)]
mod bindings;

pub use bindings::*;
