mod flow;
mod forward;
mod network;
mod reflect;
mod socket;

use std::{
    borrow::Cow,
    ffi::{c_void, CString},
    marker::PhantomData,
    mem::MaybeUninit,
    num::NonZeroU32,
};

use crate::prelude::*;
use crate::{address::WinDivertAddress, layer};
use sys::{address::WINDIVERT_ADDRESS, WinDivertParam, WinDivertShutdownMode, WINDIVERT_BATCH_MAX};
use windivert_sys as sys;

use windows::{
    core::{Error as WinError, Result as WinResult, PCSTR},
    Win32::{
        Foundation::{GetLastError, ERROR_IO_PENDING, HANDLE, WAIT_IO_COMPLETION, WAIT_TIMEOUT},
        System::{
            Services::{
                CloseServiceHandle, ControlService, OpenSCManagerA, OpenServiceA,
                SC_MANAGER_ALL_ACCESS, SERVICE_CONTROL_STOP, SERVICE_STATUS,
            },
            Threading::{CreateEventA, TlsAlloc, TlsGetValue, TlsSetValue},
            IO::{GetOverlappedResultEx, OVERLAPPED},
        },
    },
};

/// Main wrapper struct around windivert functionalities.
#[non_exhaustive]
pub struct WinDivert<L: layer::WinDivertLayerTrait> {
    handle: HANDLE,
    tls_idx: u32,
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
        let handle =
            unsafe { sys::WinDivertOpen(filter.as_ptr(), layer.into(), priority, flags.into()) };
        if handle.is_invalid() {
            let open_err = WinDivertOpenError::try_from(std::io::Error::last_os_error())?;
            Err(open_err.into())
        } else {
            Ok(Self {
                handle,
                tls_idx: windivert_tls_idx,
                _layer: PhantomData::<L>,
            })
        }
    }

    fn get_event(&self) -> Result<HANDLE, WinDivertError> {
        let mut event = HANDLE::default();
        unsafe {
            event.0 = TlsGetValue(self.tls_idx) as isize;
            if event.is_invalid() {
                event = CreateEventA(None, false, false, None)?;
                TlsSetValue(self.tls_idx, Some(event.0 as *mut c_void));
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
        };

        if res.as_bool() {
            Ok(WinDivertPacket {
                address: WinDivertAddress::<L>::from_raw(unsafe { addr.assume_init() }),
                data: buffer
                    .map(|b| Cow::Borrowed(&b[..packet_length as usize]))
                    .unwrap_or_default(),
            })
        } else {
            let recv_err = WinDivertRecvError::try_from(std::io::Error::last_os_error())?;
            Err(recv_err.into())
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
        };

        if res.as_bool() {
            addr_buffer.truncate((addr_len / ADDR_SIZE as u32) as usize);
            Ok((
                buffer.map(|buffer| &buffer[..packet_length as usize]),
                addr_buffer,
            ))
        } else {
            let recv_err = WinDivertRecvError::try_from(std::io::Error::last_os_error())?;
            Err(recv_err.into())
        }
    }

    pub(crate) fn internal_recv_wait_ex<'a>(
        &self,
        buffer: Option<&'a mut [u8]>,
        packet_count: u8,
        timeout_ms: NonZeroU32,
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
        overlapped.hEvent = self.get_event()?;

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
        };

        if !res.as_bool() && unsafe { GetLastError() } != ERROR_IO_PENDING {
            let recv_err = WinDivertRecvError::try_from(std::io::Error::last_os_error())?;
            return Err(recv_err.into());
        }

        loop {
            let res = unsafe {
                GetOverlappedResultEx(
                    self.handle,
                    &mut overlapped,
                    &mut packet_length,
                    timeout_ms.get(),
                    false,
                )
            };
            if res.as_bool() {
                break;
            }
            match unsafe { GetLastError() } {
                WAIT_IO_COMPLETION => continue,
                WAIT_TIMEOUT => {
                    return Err(WinDivertError::Timeout);
                }
                _ => {
                    let recv_err = WinDivertRecvError::try_from(std::io::Error::last_os_error())?;
                    return Err(recv_err.into());
                }
            }
        }

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
        };

        if !res.as_bool() {
            return Err(std::io::Error::last_os_error().into());
        }

        Ok(injected_length)
    }

    pub(crate) fn internal_send_ex<'packets, 'data: 'packets>(
        &self,
        packets: &'packets [WinDivertPacket<'data, L>],
    ) -> Result<u32, WinDivertError> {
        if packets.len() > WINDIVERT_BATCH_MAX as usize {
            return Err(WinDivertSendError::TooManyPackets.into());
        }
        let packet_count = packets.len();
        let mut injected_length = 0;
        let mut address_buffer: Vec<WINDIVERT_ADDRESS> = Vec::with_capacity(packet_count);
        let mut data_buffer = Vec::with_capacity(packet_count);

        for packet in packets {
            address_buffer.push(*packet.address.as_ref());
            data_buffer.push(packet.data.as_ref());
        }

        let capacity = data_buffer.iter().map(|data| data.len()).sum();
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
        };

        if !res.as_bool() {
            return Err(std::io::Error::last_os_error().into());
        }

        Ok(injected_length)
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
                Err(WinDivertError::Parameter(param, value))
            }
            _ => unsafe { sys::WinDivertSetParam(self.handle, param.into(), value) }
                .ok()
                .map_err(|_| std::io::Error::last_os_error().into()),
        }
    }

    /// Handle close function.
    pub fn close(&mut self, action: CloseAction) -> Result<(), WinDivertError> {
        let res = unsafe { sys::WinDivertClose(self.handle) };
        if !res.as_bool() {
            return Err(WinError::from(unsafe { GetLastError() }).into());
        }
        let res = unsafe { sys::WinDivertClose(self.handle) };
        if !res.as_bool() {
            return Err(WinError::from(unsafe { GetLastError() }).into());
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
}

/// Utility methods for WinDivert.
impl WinDivert<()> {
    /// Method that tries to uninstall WinDivert driver.
    pub fn uninstall() -> Result<(), WinDivertError> {
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
                return Err(WinError::from(GetLastError()).into());
            }
            let res = CloseServiceHandle(service);
            if !res.as_bool() {
                return Err(WinError::from(GetLastError()).into());
            }
            let res = CloseServiceHandle(manager);
            if !res.as_bool() {
                return Err(WinError::from(GetLastError()).into());
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
