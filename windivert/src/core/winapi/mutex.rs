use windows::{
    core::w,
    Win32::{
        Foundation::{CloseHandle, HANDLE, WAIT_ABANDONED, WAIT_OBJECT_0},
        System::Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject, INFINITE},
    },
};

#[derive(Debug)]
pub(crate) struct InstallMutex {
    handle: HANDLE,
}

impl InstallMutex {
    pub fn new() -> Result<Self, windows::core::Error> {
        let handle = unsafe { CreateMutexW(None, false, w!("WinDivertDriverInstallMutex"))? };
        Ok(Self { handle })
    }

    pub fn lock(&mut self) -> Result<InstallMutexGuard<'_>, windows::core::Error> {
        unsafe {
            match WaitForSingleObject(self.handle, INFINITE) {
                WAIT_ABANDONED | WAIT_OBJECT_0 => {}
                _ => return Err(windows::core::Error::from_win32()),
            }
        }
        Ok(InstallMutexGuard { mutex: self })
    }
}

impl Drop for InstallMutex {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.handle).expect("Handle can't be invalid") }
    }
}

#[derive(Debug)]
pub(crate) struct InstallMutexGuard<'mutex> {
    mutex: &'mutex mut InstallMutex,
}

impl<'mutex> Drop for InstallMutexGuard<'mutex> {
    fn drop(&mut self) {
        unsafe {
            ReleaseMutex(self.mutex.handle)
                .expect("Mutex is always owned by the current thread at this point");
        }
    }
}
