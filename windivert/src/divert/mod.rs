use std::{
    borrow::Cow,
    ffi::{c_void, CString},
    marker::PhantomData,
    mem::MaybeUninit,
};
use std::ffi::CStr;
use std::path::Path;

use windows::{
    core::PCSTR,
    Win32::{
        Foundation::{ERROR_IO_PENDING, ERROR_SERVICE_ALREADY_RUNNING, ERROR_SERVICE_EXISTS, HANDLE, WAIT_ABANDONED, WAIT_OBJECT_0, WAIT_TIMEOUT},
        Security::SC_HANDLE,
        System::{
            IO::{GetOverlappedResult, OVERLAPPED},
            Services::{
                CloseServiceHandle, ControlService, CreateServiceA, DeleteService,
                OpenSCManagerA, OpenServiceA, SC_MANAGER_ALL_ACCESS, SERVICE_ALL_ACCESS, SERVICE_CONTROL_STOP, SERVICE_DEMAND_START, SERVICE_ERROR_NORMAL, SERVICE_KERNEL_DRIVER, SERVICE_STATUS, StartServiceA,
            },
            Threading::{CreateEventA, CreateMutexA, INFINITE, ReleaseMutex, TlsAlloc, TlsGetValue, TlsSetValue, WaitForSingleObject},
        },
    },
};
use windows::core::{s, w};
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::{HKEY, HKEY_LOCAL_MACHINE, KEY_SET_VALUE, REG_DWORD, REG_OPTION_VOLATILE, REG_SZ, RegCloseKey, RegCreateKeyExA, RegSetValueExA, RegSetValueExW};

use windivert_sys as sys;
use sys::{address::WINDIVERT_ADDRESS, WinDivertParam, WinDivertShutdownMode};

use crate::{address::WinDivertAddress, layer};
use crate::prelude::*;

mod flow;
mod forward;
mod network;
mod reflect;
mod socket;

/// Main wrapper struct around windivert functionalities.
#[non_exhaustive]
pub struct WinDivert<L: layer::WinDivertLayerTrait> {
    handle: HANDLE,
    _tls_idx: u32,
    _layer: PhantomData<L>,
}

const ADDR_SIZE: usize = std::mem::size_of::<WINDIVERT_ADDRESS>();

impl<L: layer::WinDivertLayerTrait> WinDivert<L> {
    /// Open a handle using the specified parameters.
    fn new(
        filter: &str,
        layer: WinDivertLayer,
        priority: i16,
        flags: WinDivertFlags,
    ) -> Result<Self, WinDivertError> {
        let filter = CString::new(filter)?;
        let windivert_tls_idx = unsafe { TlsAlloc() };
        let handle = unsafe { sys::WinDivertOpen(filter.as_ptr(), layer, priority, flags) };
        if handle.is_invalid() {
            let open_err = WinDivertOpenError::try_from(std::io::Error::last_os_error())?;
            Err(WinDivertError::from(open_err))
        } else {
            Ok(Self {
                handle,
                _tls_idx: windivert_tls_idx,
                _layer: PhantomData::<L>,
            })
        }
    }

    pub(crate) fn get_event(tls_idx: u32) -> Result<HANDLE, WinDivertError> {
        let mut event = HANDLE::default();
        unsafe {
            event.0 = TlsGetValue(tls_idx) as isize;
            if event.is_invalid() {
                event = CreateEventA(None, false, false, None)?;
                TlsSetValue(tls_idx, Some(event.0 as *mut c_void))?;
            }
        }
        Ok(event)
    }

    pub(crate) fn internal_recv<'a>(
        &self,
        buffer: Option<&'a mut [u8]>,
    ) -> Result<WinDivertPacket<'a, L>, WinDivertError> {
        let mut packet_length = 0;
        let mut addr = MaybeUninit::uninit();
        let (buffer_ptr, buffer_len) = if let Some(ref buffer) = buffer {
            (buffer.as_ptr(), buffer.len())
        } else {
            (std::ptr::null(), 0)
        };

        let res = unsafe {
            sys::WinDivertRecv(
                self.handle,
                buffer_ptr as *mut c_void,
                buffer_len as u32,
                &mut packet_length,
                addr.as_mut_ptr(),
            )
        }
        .ok();

        if let Err(err) = res {
            let recv_error = WinDivertRecvError::try_from(err.code())?;
            return Err(WinDivertError::Recv(recv_error));
        }

        Ok(WinDivertPacket {
            address: WinDivertAddress::<L>::from_raw(unsafe { addr.assume_init() }),
            data: buffer
                .map(|b| Cow::Borrowed(&b[..packet_length as usize]))
                .unwrap_or_default(),
        })
    }

    pub(crate) fn internal_partial_recv<'a>(
        &self,
        buffer: Option<&'a mut [u8]>,
    ) -> Result<PacketEither<'a, L>, WinDivertError> {
        let mut packet_length = 0;
        let mut addr = MaybeUninit::uninit();
        let (buffer_ptr, buffer_len) = if let Some(ref buffer) = buffer {
            (buffer.as_ptr(), buffer.len())
        } else {
            (std::ptr::null(), 0)
        };

        let res = unsafe {
            sys::WinDivertRecv(
                self.handle,
                buffer_ptr as *mut c_void,
                buffer_len as u32,
                &mut packet_length,
                addr.as_mut_ptr(),
            )
        }
        .ok();

        let mut is_partial = false;
        if let Err(err) = res {
            let recv_error = WinDivertRecvError::try_from(err.code())?;
            if let WinDivertRecvError::InsufficientBuffer = recv_error {
                is_partial = true;
            } else {
                return Err(WinDivertError::Recv(recv_error));
            }
        }

        if is_partial {
            Ok(PacketEither::Partial(WinDivertPartialPacket {
                address: WinDivertAddress::<L>::from_raw(unsafe { addr.assume_init() }),
                data: buffer.map(|b| Cow::Borrowed(b)).unwrap_or_default(),
            }))
        } else {
            Ok(PacketEither::Full(WinDivertPacket {
                address: WinDivertAddress::<L>::from_raw(unsafe { addr.assume_init() }),
                data: buffer
                    .map(|b| Cow::Borrowed(&b[..packet_length as usize]))
                    .unwrap_or_default(),
            }))
        }
    }

    pub(crate) fn internal_recv_ex<'a>(
        &self,
        buffer: Option<&'a mut [u8]>,
        packet_count: u8,
    ) -> Result<(Option<&'a [u8]>, Vec<WINDIVERT_ADDRESS>), WinDivertError> {
        let mut packet_length = 0;

        let mut addr_len = ADDR_SIZE as u32 * packet_count as u32;
        let mut addr_buffer: Vec<WINDIVERT_ADDRESS> =
            vec![WINDIVERT_ADDRESS::default(); packet_count as usize];

        let (buffer_ptr, buffer_len) = if let Some(buffer) = &buffer {
            (buffer.as_ptr(), buffer.len())
        } else {
            (std::ptr::null(), 0)
        };

        let res = unsafe {
            sys::WinDivertRecvEx(
                self.handle,
                buffer_ptr as *mut c_void,
                buffer_len as u32,
                &mut packet_length,
                0,
                addr_buffer.as_mut_ptr(),
                &mut addr_len,
                std::ptr::null_mut(),
            )
        }
        .ok();

        if let Err(err) = res {
            let recv_error = WinDivertRecvError::try_from(err.code())?;
            return Err(WinDivertError::Recv(recv_error));
        }

        addr_buffer.truncate((addr_len / ADDR_SIZE as u32) as usize);
        Ok((
            buffer.map(|buffer| &buffer[..packet_length as usize]),
            addr_buffer,
        ))
    }

    pub(crate) fn internal_recv_wait_ex<'a>(
        &self,
        buffer: Option<&'a mut [u8]>,
        packet_count: u8,
        timeout_ms: u32,
    ) -> Result<(Option<&'a [u8]>, Vec<WINDIVERT_ADDRESS>), WinDivertError> {
        let mut packet_length = 0;

        let mut addr_len = ADDR_SIZE as u32 * packet_count as u32;
        let mut addr_buffer: Vec<WINDIVERT_ADDRESS> =
            vec![WINDIVERT_ADDRESS::default(); packet_count as usize];

        let (buffer_ptr, buffer_len) = if let Some(buffer) = &buffer {
            (buffer.as_ptr(), buffer.len())
        } else {
            (std::ptr::null(), 0)
        };

        let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };
        overlapped.hEvent = Self::get_event(self._tls_idx)?;

        let res = unsafe {
            sys::WinDivertRecvEx(
                self.handle,
                buffer_ptr as *mut c_void,
                buffer_len as u32,
                &mut packet_length,
                0,
                addr_buffer.as_mut_ptr(),
                &mut addr_len,
                &mut overlapped,
            )
        }
            .ok();

        if let Err(err) = res {
            if err.code() != ERROR_IO_PENDING.to_hresult() {
                let recv_error = WinDivertRecvError::try_from(err.code())?;
                return Err(WinDivertError::Recv(recv_error));
            }
        }

        match unsafe { WaitForSingleObject(overlapped.hEvent, timeout_ms) } {
            WAIT_OBJECT_0 => {}
            WAIT_TIMEOUT => {
                return Err(WinDivertError::Timeout);
            }
            _ => {
                let recv_error =
                    WinDivertRecvError::try_from(windows::core::Error::from_win32().code())?;
                return Err(recv_error.into());
            }
        }

        unsafe { GetOverlappedResult(self.handle, &overlapped, &mut packet_length, false) }?;

        addr_buffer.truncate((addr_len / ADDR_SIZE as u32) as usize);
        Ok((
            buffer.map(|buffer| &buffer[..packet_length as usize]),
            addr_buffer,
        ))
    }

    pub(crate) fn internal_send(&self, packet: &WinDivertPacket<L>) -> Result<u32, WinDivertError> {
        let mut injected_length = 0;

        let res = unsafe {
            sys::WinDivertSend(
                self.handle,
                packet.data.as_ptr() as *const c_void,
                packet.data.len() as u32,
                &mut injected_length,
                packet.address.as_ref(),
            )
        }
        .ok();

        if let Err(err) = res {
            let send_error = WinDivertSendError::try_from(err.code())?;
            return Err(WinDivertError::Send(send_error));
        }

        Ok(injected_length)
    }

    pub(crate) fn internal_send_ex<'packets, 'data: 'packets>(
        &self,
        packets: &'packets [WinDivertPacket<'data, L>],
    ) -> Result<u32, WinDivertError> {
        if packets.len() > WinDivert::MAX_BATCH as usize {
            return Err(WinDivertError::from(WinDivertSendError::TooManyPackets));
        }
        let packet_count = packets.len();
        let mut injected_length = 0;
        let mut address_buffer: Vec<WINDIVERT_ADDRESS> = Vec::with_capacity(packet_count);
        let mut data_buffer = Vec::with_capacity(packet_count);

        let mut capacity = 0;
        for packet in packets {
            address_buffer.push(*packet.address.as_ref());
            data_buffer.push(packet.data.as_ref());
            capacity += packet.data.len();
        }

        let mut packet_buffer: Vec<u8> = Vec::with_capacity(capacity);
        for data in data_buffer {
            packet_buffer.extend(data.iter());
        }

        let res = unsafe {
            sys::WinDivertSendEx(
                self.handle,
                packet_buffer.as_ptr() as *const c_void,
                packet_buffer.len() as u32,
                &mut injected_length,
                0,
                address_buffer.as_ptr(),
                (ADDR_SIZE * packet_count) as u32,
                std::ptr::null_mut(),
            )
        }
        .ok();

        if let Err(err) = res {
            let send_error = WinDivertSendError::try_from(err.code())?;
            return Err(WinDivertError::Send(send_error));
        }

        Ok(injected_length)
    }

    /// Methods that allows to query the driver for parameters.
    pub fn get_param(&self, param: WinDivertParam) -> Result<u64, WinDivertError> {
        let mut value = 0;
        unsafe { sys::WinDivertGetParam(self.handle, param, &mut value) }.ok()?;
        Ok(value)
    }

    /// Method that allows setting driver parameters.
    pub fn set_param(&self, param: WinDivertParam, value: u64) -> Result<(), WinDivertError> {
        match param {
            WinDivertParam::VersionMajor | WinDivertParam::VersionMinor => {
                return Err(WinDivertError::Parameter(param, value));
            }
            _ => unsafe { sys::WinDivertSetParam(self.handle, param, value) }.ok()?,
        }
        Ok(())
    }

    /// Handle close function.
    pub fn close(&mut self, action: CloseAction) -> Result<(), WinDivertError> {
        unsafe { sys::WinDivertClose(self.handle) }.ok()?;
        match action {
            CloseAction::Uninstall => WinDivert::uninstall(),
            CloseAction::Nothing => Ok(()),
        }
    }

    /// Shutdown function.
    pub fn shutdown(&mut self, mode: WinDivertShutdownMode) -> windows::core::Result<()> {
        unsafe { sys::WinDivertShutdown(self.handle, mode) }.ok()
    }
}

/// Utility methods for WinDivert.
impl WinDivert<()> {
    /// Maximum number of packets that can be captured/sent in a single batched operation
    pub const MAX_BATCH: u8 = windivert_sys::WINDIVERT_BATCH_MAX as u8;
    const WINDIVERT_DEVICE_NAME: PCSTR = s!("WinDivert");

    /// Method to manually install WinDivert driver.
    /// This method allow installing a sys file not located on the same folder as the dll
    pub fn install(path: &Path) -> Result<(), WinDivertError> {
        let path_str = CString::new(path.as_os_str().as_encoded_bytes())?;
        let mut result = Ok(());

        let mutex = unsafe {
            let mutex = CreateMutexA(None, false, s!("WinDivertDriverInstallMutex"))?;

            match WaitForSingleObject(mutex, INFINITE) {
                WAIT_ABANDONED | WAIT_OBJECT_0 => {}
                _ => return Err(WinDivertError::OSError(windows::core::Error::from_win32()))
            }

            mutex
        };

        if let Ok(manager) = unsafe { OpenSCManagerA(None, None, SC_MANAGER_ALL_ACCESS) } {
            // Check if service exists
            let mut service: Option<SC_HANDLE> = unsafe { OpenServiceA(manager, Self::WINDIVERT_DEVICE_NAME, SERVICE_ALL_ACCESS).ok() };
            // Create service if not exists
            if service.is_none() {
                match unsafe {
                    CreateServiceA(
                        manager,
                        Self::WINDIVERT_DEVICE_NAME,
                        Self::WINDIVERT_DEVICE_NAME,
                        SERVICE_ALL_ACCESS,
                        SERVICE_KERNEL_DRIVER,
                        SERVICE_DEMAND_START,
                        SERVICE_ERROR_NORMAL,
                        PCSTR::from_raw(path_str.as_bytes().as_ptr()),
                        None,
                        None,
                        None,
                        None,
                        None,
                    )
                } {
                    Ok(service_handle) => service = Some(service_handle),
                    Err(err) => {
                        if err.code() == ERROR_SERVICE_EXISTS.to_hresult() {
                            service = match unsafe { OpenServiceA(manager, Self::WINDIVERT_DEVICE_NAME, SERVICE_ALL_ACCESS) } {
                                Ok(service) => Some(service),
                                Err(err) => {
                                    result = Err(WinDivertError::OSError(err));
                                    None
                                }
                            }
                        }
                    }
                };
            }

            // Register event logging and start service
            if let Some(service) = service.as_ref() {
                let _ = register_event_source(path_str.as_c_str());

                result = match unsafe { StartServiceA(*service, None) } {
                    Ok(_) => Ok(()),
                    Err(err) => {
                        if err.code() == ERROR_SERVICE_ALREADY_RUNNING.to_hresult() {
                            Ok(())
                        } else {
                            Err(WinDivertError::OSError(err))
                        }
                    }
                };

                if result.is_ok() {
                    let _ = unsafe { DeleteService(*service) };
                }
            }


            unsafe { CloseServiceHandle(manager).expect("Manager can't be invalid") };
        }

        unsafe { ReleaseMutex(mutex).expect("Mutex is always owned by the current thread at this point") };
        result
    }

    /// Method that tries to uninstall WinDivert driver.
    pub fn uninstall() -> Result<(), WinDivertError> {
        let status: *mut SERVICE_STATUS = MaybeUninit::uninit().as_mut_ptr();
        unsafe {
            let manager = OpenSCManagerA(None, None, SC_MANAGER_ALL_ACCESS)?;
            let service = OpenServiceA(
                manager,
                Self::WINDIVERT_DEVICE_NAME,
                SC_MANAGER_ALL_ACCESS,
            )?;
            ControlService(service, SERVICE_CONTROL_STOP, status)?;
            CloseServiceHandle(service)?;
            CloseServiceHandle(manager)?;
        }
        Ok(())
    }
}

fn register_event_source(path_str: &CStr) -> Result<(), windows::core::Error> {
    let key = unsafe {
        let mut key: MaybeUninit<HKEY> = MaybeUninit::uninit();
        if let Err(err) = RegCreateKeyExA(
            HKEY_LOCAL_MACHINE,
            s!("System\\CurrentControlSet\\Services\\EventLog\\System\\WinDivert"),
            0,
            None,
            REG_OPTION_VOLATILE,
            KEY_SET_VALUE,
            None,
            key.as_mut_ptr(),
            None,
        ) {
            if err.code() != ERROR_SUCCESS.to_hresult() {
                return Err(err)
            }
        };

        key.assume_init()
    };

    let _ = unsafe { RegSetValueExW(key, w!("EventMessageFile"), 0, REG_SZ, Some(path_str.to_bytes())) };
    let _ = unsafe { RegSetValueExA(key, s!("TypesSupported"), 0, REG_DWORD, Some(&7u32.to_le_bytes())) };

    unsafe { RegCloseKey(key)? };

    Ok(())
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
