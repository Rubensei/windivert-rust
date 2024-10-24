use std::{
    mem::MaybeUninit,
    os::windows::ffi::OsStrExt,
    path::{Path, PathBuf},
};

use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{
            ERROR_SERVICE_CANNOT_ACCEPT_CTRL, ERROR_SERVICE_DOES_NOT_EXIST, ERROR_SERVICE_EXISTS,
            ERROR_SERVICE_NOT_ACTIVE, WIN32_ERROR,
        },
        System::{
            Registry::{
                RegCloseKey, RegCreateKeyExW, RegSetValueExW, HKEY, HKEY_LOCAL_MACHINE,
                KEY_SET_VALUE, REG_DWORD, REG_OPTION_VOLATILE, REG_SZ,
            },
            Services::{
                CloseServiceHandle, ControlService, CreateServiceW, DeleteService, OpenServiceW,
                StartServiceW, SC_HANDLE, SERVICE_ALL_ACCESS, SERVICE_CONTROL_STOP,
                SERVICE_DEMAND_START, SERVICE_ERROR_NORMAL, SERVICE_KERNEL_DRIVER, SERVICE_STATUS,
            },
        },
    },
};

use crate::error;

use super::sc_manager::ScManager;

#[derive(Debug)]
pub(crate) struct Service {
    handle: SC_HANDLE,
    path: PathBuf,
}

impl Service {
    const WINDIVERT_DEVICE_NAME: PCWSTR = w!("WinDivert");

    pub fn open(manager: &ScManager) -> Result<Self, windows::core::Error> {
        let handle = unsafe {
            OpenServiceW(
                SC_HANDLE::from(manager),
                Self::WINDIVERT_DEVICE_NAME,
                SERVICE_ALL_ACCESS,
            )?
        };
        Ok(Self {
            handle,
            path: PathBuf::with_capacity(0),
        })
    }

    pub fn new(manager: &ScManager, path: &Path) -> Result<Self, windows::core::Error> {
        let handle = unsafe {
            match OpenServiceW(
                SC_HANDLE::from(manager),
                Self::WINDIVERT_DEVICE_NAME,
                SERVICE_ALL_ACCESS,
            ) {
                Ok(service) => service,
                Err(error) => {
                    if let Some(ERROR_SERVICE_DOES_NOT_EXIST) = WIN32_ERROR::from_error(&error) {
                        Self::create_service(manager, path)?
                    } else {
                        return Err(error);
                    }
                }
            }
        };

        Ok(Self {
            handle,
            path: path.to_path_buf(),
        })
    }

    fn create_service(manager: &ScManager, path: &Path) -> Result<SC_HANDLE, windows::core::Error> {
        let path_str = path
            .as_os_str()
            .encode_wide()
            .into_iter()
            .chain([0])
            .collect::<Vec<u16>>();

        unsafe {
            match CreateServiceW(
                SC_HANDLE::from(manager),
                Self::WINDIVERT_DEVICE_NAME,
                Self::WINDIVERT_DEVICE_NAME,
                SERVICE_ALL_ACCESS,
                SERVICE_KERNEL_DRIVER,
                SERVICE_DEMAND_START,
                SERVICE_ERROR_NORMAL,
                PCWSTR::from_raw(path_str.as_ptr()),
                None,
                None,
                None,
                None,
                None,
            ) {
                Ok(handle) => Ok(handle),
                Err(error) => {
                    if let Some(ERROR_SERVICE_EXISTS) = WIN32_ERROR::from_error(&error) {
                        OpenServiceW(
                            SC_HANDLE::from(manager),
                            Self::WINDIVERT_DEVICE_NAME,
                            SERVICE_ALL_ACCESS,
                        )
                    } else {
                        Err(error)
                    }
                }
            }
        }
    }

    pub fn register_event_source(&self) -> Result<(), windows::core::Error> {
        unsafe {
            let key = {
                let mut key: MaybeUninit<HKEY> = MaybeUninit::uninit();
                RegCreateKeyExW(
                    HKEY_LOCAL_MACHINE,
                    w!("System\\CurrentControlSet\\Services\\EventLog\\System\\WinDivert"),
                    0,
                    None,
                    REG_OPTION_VOLATILE,
                    KEY_SET_VALUE,
                    None,
                    key.as_mut_ptr(),
                    None,
                )
                .ok()?;

                key.assume_init()
            };

            RegSetValueExW(
                key,
                w!("EventMessageFile"),
                0,
                REG_SZ,
                Some(self.path.as_os_str().as_encoded_bytes()),
            )
            .ok()?;
            RegSetValueExW(
                key,
                w!("TypesSupported"),
                0,
                REG_DWORD,
                Some(&7u32.to_le_bytes()),
            )
            .ok()?;

            RegCloseKey(key).ok()?;
        }
        Ok(())
    }

    pub fn start_and_mark_for_deletion(&self) -> Result<(), windows::core::Error> {
        unsafe {
            match StartServiceW(self.handle, None) {
                Ok(_) => DeleteService(self.handle),
                Err(error) => {
                    if let Some(ERROR_SERVICE_ALREADY_RUNNING) = WIN32_ERROR::from_error(&error) {
                        Ok(())
                    } else {
                        Err(error)
                    }
                }
            }
        }
    }

    pub fn stop(&self) -> Result<(), windows::core::Error> {
        unsafe {
            match ControlService(self.handle, SERVICE_CONTROL_STOP, std::ptr::null_mut()) {
                Err(error) => {
                    // The only scenario when a ControlService(SERVICE_CONTROL_STOP)  raises ERROR_SERVICE_CANNOT_ACCEPT_CTRL is if the service is STOP_PENDING
                    // It's safe to treat it as a success due to how this method is used
                    if let Some(ERROR_SERVICE_NOT_ACTIVE | ERROR_SERVICE_CANNOT_ACCEPT_CTRL) =
                        WIN32_ERROR::from_error(&error)
                    {
                        Ok(())
                    } else {
                        Err(error)
                    }
                }
                _ => Ok(()),
            }
        }
    }
}

impl From<Service> for SC_HANDLE {
    fn from(value: Service) -> Self {
        value.handle
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        unsafe { CloseServiceHandle(self.handle).expect("Handle can't be invalid") }
    }
}
