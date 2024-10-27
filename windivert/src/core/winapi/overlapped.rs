use std::{ffi::c_void, mem::ManuallyDrop};

use windows::Win32::{
    Foundation::{DuplicateHandle, DUPLICATE_SAME_ACCESS, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT},
    System::{
        Threading::{GetCurrentProcess, WaitForSingleObject},
        IO::{CancelIo, GetOverlappedResult, OVERLAPPED},
    },
};

use super::tls::TlsIndex;

#[cfg(test)]
use mockall::automock;

pub(crate) struct Overlapped {
    inner: OVERLAPPED,
    handle: HANDLE,
}

#[cfg_attr(test, automock)]
impl Overlapped {
    pub fn init(handle: &HANDLE, tls_index: &TlsIndex) -> Result<Self, windows::core::Error> {
        Ok(Self {
            inner: unsafe {
                let mut inner = OVERLAPPED::default();
                inner.hEvent = tls_index.get_or_init_event()?;
                inner
            },
            // SAFETY
            // This is safe since the cloned handle is only used internally during a single overlapped io that will block the thread
            handle: handle.clone(),
        })
    }

    pub fn as_raw_mut(&mut self) -> *mut c_void {
        &mut self.inner as *mut OVERLAPPED as *mut c_void
    }

    /// Methods that waits until the overlapped event is signaled
    /// It will return `Some` if the operation completed, and `None` if the timeout expired
    pub fn wait_for_object(&self, timeout_ms: u32) -> Result<Option<()>, windows::core::Error> {
        unsafe {
            match WaitForSingleObject(self.inner.hEvent, timeout_ms) {
                WAIT_OBJECT_0 => Ok(Some(())),
                WAIT_TIMEOUT => {
                    CancelIo(self.handle)?;
                    Ok(None)
                }
                _ => Err(windows::core::Error::from_win32()),
            }
        }
    }

    pub fn get_result(self) -> Result<u32, windows::core::Error> {
        let mut written_bytes: u32 = 0;
        unsafe { GetOverlappedResult(self.handle, &self.inner, &mut written_bytes, false)? };
        Ok(written_bytes)
    }

    pub fn cancel(self) -> Result<(), windows::core::Error> {
        unsafe { CancelIo(self.handle) }
    }
}
