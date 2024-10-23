use windows::{
    core::w,
    Win32::{
        Foundation::{HANDLE, WAIT_ABANDONED, WAIT_OBJECT_0},
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

    pub fn lock(&mut self) -> Result<InstallMutexGuard, windows::core::Error> {
        unsafe {
            match WaitForSingleObject(self.handle, INFINITE) {
                WAIT_ABANDONED | WAIT_OBJECT_0 => {}
                _ => return Err(windows::core::Error::from_win32()),
            }
        }
        Ok(InstallMutexGuard {
            mutex: &mut self.handle,
        })
    }
}

#[derive(Debug)]
pub(crate) struct InstallMutexGuard<'mutex> {
    mutex: &'mutex mut HANDLE,
}

impl<'mutex> Drop for InstallMutexGuard<'mutex> {
    fn drop(&mut self) {
        unsafe {
            ReleaseMutex(*self.mutex)
                .expect("Mutex is always owned by the current thread at this point");
        }
    }
}
