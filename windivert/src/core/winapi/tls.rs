use windows::Win32::{
    Foundation::HANDLE,
    System::Threading::{CreateEventW, TlsAlloc, TlsFree, TlsGetValue, TlsSetValue},
};

/// Wrapper around used TLS operations
#[derive(Debug, Clone)]
pub struct TlsIndex {
    index: u32,
}

impl TlsIndex {
    pub(crate) fn alloc_tls() -> windows::core::Result<TlsIndex> {
        match unsafe { TlsAlloc() } {
            u32::MAX => Err(windows::core::Error::from_win32()),
            index => Ok(TlsIndex { index }),
        }
    }

    pub(crate) fn get_or_init_event(&self) -> windows::core::Result<HANDLE> {
        let mut event = HANDLE::default();
        unsafe {
            event.0 = TlsGetValue(self.index);
            if event.is_invalid() {
                event = CreateEventW(None, false, false, None)?;
                TlsSetValue(self.index, Some(event.0))?;
            }
        }
        Ok(event)
    }
}

impl Drop for TlsIndex {
    fn drop(&mut self) {
        let _ = unsafe { TlsFree(self.index) };
    }
}

#[cfg(test)]
mod test {
    use super::TlsIndex;

    #[test]
    fn get_event_twice_returns_same_event() {
        let tls = TlsIndex::alloc_tls();
        assert!(tls.is_ok());
        let tls = tls.unwrap();
        let event_a = tls.get_or_init_event();
        let event_b = tls.get_or_init_event();
        assert_eq!(event_a, event_b);
    }
}
