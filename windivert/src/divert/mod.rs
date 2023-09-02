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
};

use crate::prelude::*;
use crate::{address::WinDivertAddress, layer};
use sys::{address::WINDIVERT_ADDRESS, WinDivertParam, WinDivertShutdownMode};
use windivert_sys as sys;

use windows::{
    core::{Result as WinResult, PCSTR},
    Win32::{
        Foundation::HANDLE,
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
            Err(open_err.into())
        } else {
            Ok(Self {
                handle,
                _tls_idx: windivert_tls_idx,
                _layer: PhantomData::<L>,
            })
        }
    }

    pub(crate) fn _get_event(tls_idx: u32) -> Result<HANDLE, WinDivertError> {
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
                return Err(WinDivertError::Parameter(param, value))
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
    pub fn shutdown(&mut self, mode: WinDivertShutdownMode) -> WinResult<()> {
        unsafe { sys::WinDivertShutdown(self.handle, mode) }.ok()?;
        Ok(())
    }
}

/// Utility methods for WinDivert.
impl WinDivert<()> {
    /// Maximum number of packets that can be captured/sent in a single batched operation
    pub const MAX_BATCH: u8 = windivert_sys::WINDIVERT_BATCH_MAX as u8;

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
            ControlService(service, SERVICE_CONTROL_STOP, status)?;
            CloseServiceHandle(service)?;
            CloseServiceHandle(manager)?;
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
