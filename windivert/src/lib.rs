#![deny(missing_docs)]
/*!
Wrapper around [`windivert_sys`] ffi crate.
*/

/// Module containing abstractions of core low level apis to enable mocking the blocking operations and test the remaining code
pub(crate) mod core;

/// WinDivert address data structures
pub mod address;
mod divert;
/// WinDivert error types
pub mod error;
/// Layer types used for typestate pattern
pub mod layer;
/// WinDivert packet types
pub mod packet;

pub use divert::*;

mod utils;

/// Prelude module for [`WinDivert`].
pub mod prelude {
    pub use windivert_sys::{
        WinDivertEvent, WinDivertFlags, WinDivertLayer, WinDivertParam, WinDivertShutdownMode,
    };

    pub use crate::address::*;
    pub use crate::divert::*;
    pub use crate::error::*;
    pub use crate::layer::*;
    pub use crate::packet::*;
}

pub(crate) mod test_data;
