#![deny(missing_docs)]
/*!
Wrapper around [`windivert_sys`] ffi crate.

# Blocking operations
Since all the recv/send methods are blocking, usually the filtering/sniffing should be done in a separate thread or a graceful shutdown might not be possible.
The `shutdown()` method can be used anytime to stop gracefully any ongoing operations, but will prevent any further recv/send calls using the same handle.
For cases where a handle might need to be reused, `_wait` variants of the methods are provided.
These variants will wait at most the specified duration for the operation to complete, allowing the filtering thread to respond to other events in a timely manner.

# Example
```no_run
use windivert::prelude::*;

let Ok(divert) = WinDivert::network("ip and tcp.DstPort == 443", 0, Default::default()) else {
    panic!("Failed to create WinDivert");
};

let divert = std::sync::Arc::new(divert);

let divert_shared = divert.clone();
let handle = std::thread::spawn(move || {
    // Do something in the background
    let mut buffer = [0u8; 1500];

    loop {
        match divert_shared.recv(&mut buffer) {
            Ok(packet) => {
                // In capture mode the packet is captured and not calling `send()` with it will prevent it from reaching the destination.
                divert_shared.send(&packet).expect("Failed to send packet");
            },
            Err(WinDivertError::Recv(WinDivertRecvError::NoData)) => {
                // Handle was shutdown, and there is no more pending data to receive
                break;
            },
            Err(e) => {
                // Other errors
                eprintln!("Error receiving packet: {}", e);
            }
        }
    }
});

std::thread::sleep(std::time::Duration::from_secs(10));

divert
    .shutdown(WinDivertShutdownMode::Both)
    .expect("Failed to shutdown WinDivert");

handle.join().unwrap();

std::sync::Arc::try_unwrap(divert)
    .expect("Thread already finished, no references remaining")
    .close(CloseAction::Nothing)
    .expect("Failed to close WinDivert");
```
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
