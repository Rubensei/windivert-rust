use std::ffi::CString;

use crate::prelude::*;
use windivert_sys as sys;

use windows::Win32::System::Threading::TlsAlloc;

/// Builder struct for WinDivert
pub struct WinDivertBuilder {
    filter: String,
    layer: WinDivertLayer,
    priority: i16,
    flags: WinDivertFlags,
}

impl WinDivert {
    /// Init windivert builder.
    pub fn builder(filter: &str, layer: WinDivertLayer) -> WinDivertBuilder {
        WinDivertBuilder {
            filter: filter.to_string(),
            layer,
            priority: Default::default(),
            flags: Default::default(),
        }
    }
}

impl WinDivertBuilder {
    /// Priority setter
    pub fn priority(self, priority: i16) -> Self {
        Self { priority, ..self }
    }

    /// Flags setter
    pub fn flags(self, flags: WinDivertFlags) -> Self {
        Self { flags, ..self }
    }

    /// Builder build method
    pub fn build(self) -> Result<WinDivert, WinDivertError> {
        let filter = CString::new(self.filter)?;
        let windivert_tls_idx = unsafe { TlsAlloc() };
        let handle = unsafe {
            sys::WinDivertOpen(
                filter.as_ptr(),
                self.layer.into(),
                self.priority,
                self.flags.into(),
            )
        };
        if handle.is_invalid() {
            match WinDivertOpenError::try_from(std::io::Error::last_os_error()) {
                Ok(err) => Err(WinDivertError::Open(err)),
                Err(err) => Err(WinDivertError::OSError(err)),
            }
        } else {
            Ok(WinDivert {
                handle,
                layer: self.layer,
                tls_idx: windivert_tls_idx,
            })
        }
    }
}
