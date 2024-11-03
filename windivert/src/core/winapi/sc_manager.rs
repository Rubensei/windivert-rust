use windows::Win32::System::Services::{
    CloseServiceHandle, OpenSCManagerW, SC_HANDLE, SC_MANAGER_ALL_ACCESS,
};

#[derive(Debug)]
pub(crate) struct ScManager {
    handle: SC_HANDLE,
}

impl ScManager {
    pub fn new() -> Result<Self, windows::core::Error> {
        let handle = unsafe { OpenSCManagerW(None, None, SC_MANAGER_ALL_ACCESS)? };
        Ok(Self { handle })
    }
}

impl From<&ScManager> for SC_HANDLE {
    fn from(value: &ScManager) -> Self {
        value.handle
    }
}

impl Drop for ScManager {
    fn drop(&mut self) {
        unsafe {
            CloseServiceHandle(self.handle).expect("Handle can't be invalid");
        }
    }
}
