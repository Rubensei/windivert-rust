mod blocking;
mod builder;

use std::{
    ffi::{c_void, CString},
    mem::MaybeUninit,
};

use crate::prelude::*;
use sys::{WinDivertParam, WinDivertShutdownMode};
use windivert_sys as sys;

use windows::{
    core::{Error as WinError, Result as WinResult, PCSTR},
    Win32::{
        Foundation::{GetLastError, HANDLE},
        System::{
            Services::{
                CloseServiceHandle, ControlService, OpenSCManagerA, OpenServiceA,
                SC_MANAGER_ALL_ACCESS, SERVICE_CONTROL_STOP, SERVICE_STATUS,
            },
            Threading::{CreateEventA, TlsAlloc, TlsGetValue, TlsSetValue},
        },
    },
};

/// Main wrapper struct around windivert functionalities.
pub struct WinDivert {
    handle: HANDLE,
    layer: WinDivertLayer,
    tls_idx: u32,
}

/// Recv implementations
impl WinDivert {
    /// Open a handle using the specified parameters.
    pub fn new(
        filter: &str,
        layer: WinDivertLayer,
        priority: i16,
        flags: WinDivertFlags,
    ) -> Result<Self, WinDivertError> {
        let filter = CString::new(filter)?;
        let windivert_tls_idx = unsafe { TlsAlloc() };
        let handle =
            unsafe { sys::WinDivertOpen(filter.as_ptr(), layer.into(), priority, flags.into()) };
        if handle.is_invalid() {
            match WinDivertOpenError::try_from(std::io::Error::last_os_error()) {
                Ok(err) => Err(WinDivertError::Open(err)),
                Err(err) => Err(WinDivertError::OSError(err)),
            }
        } else {
            Ok(Self {
                handle,
                layer,
                tls_idx: windivert_tls_idx,
            })
        }
    }

    pub(crate) fn get_event(tls_idx: u32) -> Result<HANDLE, WinDivertError> {
        let mut event = HANDLE::default();
        unsafe {
            event.0 = TlsGetValue(tls_idx) as isize;
            if event.is_invalid() {
                event = CreateEventA(None, false, false, None)?;
                TlsSetValue(tls_idx, Some(event.0 as *mut c_void));
            }
        }
        Ok(event)
    }

    /// Methods that allows to query the driver for parameters.
    pub fn get_param(&self, param: WinDivertParam) -> Result<u64, WinDivertError> {
        let mut value = 0;
        let res = unsafe { sys::WinDivertGetParam(self.handle, param.into(), &mut value) };
        if !res.as_bool() {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(value)
    }

    /// Method that allows setting driver parameters.
    pub fn set_param(&self, param: WinDivertParam, value: u64) -> Result<(), WinDivertError> {
        match param {
            WinDivertParam::VersionMajor | WinDivertParam::VersionMinor => {
                Err(WinDivertError::Parameter)
            }
            _ => unsafe { sys::WinDivertSetParam(self.handle, param.into(), value) }
                .ok()
                .map_err(|_| std::io::Error::last_os_error().into()),
        }
    }

    /// Handle close function.
    pub fn close(&mut self, action: CloseAction) -> WinResult<()> {
        let res = unsafe { sys::WinDivertClose(self.handle) };
        if !res.as_bool() {
            return Err(WinError::from(unsafe { GetLastError() }));
        }
        let res = unsafe { sys::WinDivertClose(self.handle) };
        if !res.as_bool() {
            return Err(WinError::from(unsafe { GetLastError() }));
        }
        match action {
            CloseAction::Uninstall => WinDivert::uninstall(),
            CloseAction::Nothing => Ok(()),
        }
    }

    /// Shutdown function.
    pub fn shutdown(&mut self, mode: WinDivertShutdownMode) -> WinResult<()> {
        let res = unsafe { sys::WinDivertShutdown(self.handle, mode.into()) };
        if !res.as_bool() {
            return Err(WinError::from(unsafe { GetLastError() }));
        }
        Ok(())
    }

    /// Method that tries to uninstall WinDivert driver.
    pub fn uninstall() -> WinResult<()> {
        let status: *mut SERVICE_STATUS = MaybeUninit::uninit().as_mut_ptr();
        unsafe {
            let manager = OpenSCManagerA(None, None, SC_MANAGER_ALL_ACCESS)?;
            let service = OpenServiceA(
                manager,
                PCSTR::from_raw("WinDivert".as_ptr()),
                SC_MANAGER_ALL_ACCESS,
            )?;
            let res = ControlService(service, SERVICE_CONTROL_STOP, status);
            if !res.as_bool() {
                return Err(WinError::from(GetLastError()));
            }
            let res = CloseServiceHandle(service);
            if !res.as_bool() {
                return Err(WinError::from(GetLastError()));
            }
            let res = CloseServiceHandle(manager);
            if !res.as_bool() {
                return Err(WinError::from(GetLastError()));
            }
        }
        Ok(())
    }
}

/// Action parameter for  [`WinDivert::close()`](`fn@WinDivert::close`)
pub enum CloseAction {
    /// Close the handle and try to uninstall the WinDivert driver.
    Uninstall,
    /// Close the handle without uninstalling the driver.
    Nothing,
}

impl Default for CloseAction {
    fn default() -> Self {
        Self::Nothing
    }
}
