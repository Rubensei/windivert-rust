use std::{
    borrow::Cow,
    ffi::{c_void, CString},
    marker::PhantomData,
    mem::MaybeUninit,
    path::Path,
};

use windows::Win32::Foundation::{
    BOOL, ERROR_IO_PENDING, ERROR_SERVICE_DOES_NOT_EXIST, HANDLE, WIN32_ERROR,
};

use windivert_sys::{
    address::WINDIVERT_ADDRESS, WinDivertClose, WinDivertGetParam, WinDivertParam,
    WinDivertSetParam, WinDivertShutdown, WinDivertShutdownMode,
};

#[cfg(test)]
use crate::core::MockSysWrapper as SysWrapper;

#[cfg(not(test))]
use crate::core::SysWrapper;

#[cfg(test)]
use crate::core::winapi::overlapped::MockOverlapped as Overlapped;

#[cfg(not(test))]
use crate::core::winapi::overlapped::Overlapped;

use crate::core::winapi::{
    mutex::InstallMutex, sc_manager::ScManager, service::WinDivertDriverService, tls::TlsIndex,
};
use crate::layer;
use crate::prelude::*;

mod flow;
mod forward;
mod network;
mod reflect;
mod socket;

/// Main wrapper struct around windivert functionalities.
#[non_exhaustive]
#[derive(Debug)]
pub struct WinDivert<L: layer::WinDivertLayerTrait> {
    handle: HANDLE,
    tls_index: TlsIndex,
    core: SysWrapper,
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
        let windivert_tls_idx = TlsIndex::alloc_tls()?;
        let sys_wrapper = SysWrapper::new();
        let handle =
            unsafe { HANDLE(sys_wrapper.WinDivertOpen(filter.as_ptr(), layer, priority, flags)) };
        if handle.is_invalid() {
            let open_err = WinDivertOpenError::try_from(windows::core::Error::from_win32())?;
            Err(open_err.into())
        } else {
            Ok(Self {
                handle,
                tls_index: windivert_tls_idx,
                core: sys_wrapper,
                _layer: PhantomData::<L>,
            })
        }
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
            BOOL(self.core.WinDivertRecv(
                self.handle.0,
                buffer_ptr as *mut c_void,
                buffer_len as u32,
                &mut packet_length,
                addr.as_mut_ptr(),
            ))
        }
        .ok();

        if let Err(err) = res {
            let recv_error = WinDivertRecvError::try_from(err)?;
            return Err(recv_error.into());
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
            BOOL(self.core.WinDivertRecv(
                self.handle.0,
                buffer_ptr as *mut c_void,
                buffer_len as u32,
                &mut packet_length,
                addr.as_mut_ptr(),
            ))
        }
        .ok();

        let mut is_partial = false;
        if let Err(err) = res {
            let recv_error = WinDivertRecvError::try_from(err)?;
            if let WinDivertRecvError::InsufficientBuffer = recv_error {
                is_partial = true;
            } else {
                return Err(recv_error.into());
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
            BOOL(self.core.WinDivertRecvEx(
                self.handle.0,
                buffer_ptr as *mut c_void,
                buffer_len as u32,
                &mut packet_length,
                0,
                addr_buffer.as_mut_ptr(),
                &mut addr_len,
                std::ptr::null_mut(),
            ))
        }
        .ok();

        if let Err(err) = res {
            let recv_error = WinDivertRecvError::try_from(err)?;
            return Err(recv_error.into());
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
        let mut addr_len = ADDR_SIZE as u32 * packet_count as u32;
        let mut addr_buffer: Vec<WINDIVERT_ADDRESS> =
            vec![WINDIVERT_ADDRESS::default(); packet_count as usize];

        let (buffer_ptr, buffer_len) = if let Some(buffer) = &buffer {
            (buffer.as_ptr(), buffer.len())
        } else {
            (std::ptr::null(), 0)
        };

        let mut overlapped = Overlapped::init(&self.handle, &self.tls_index)?;

        let res = unsafe {
            BOOL(self.core.WinDivertRecvEx(
                self.handle.0,
                buffer_ptr as *mut c_void,
                buffer_len as u32,
                std::ptr::null_mut(),
                0,
                addr_buffer.as_mut_ptr(),
                &mut addr_len,
                overlapped.as_raw_mut(),
            ))
        }
        .ok();

        if let Err(err) = res {
            if err.code() != ERROR_IO_PENDING.to_hresult() {
                overlapped.cancel()?;
                let recv_error = WinDivertRecvError::try_from(err)?;
                return Err(recv_error.into());
            }
        }

        let Some(_) = overlapped.wait_for_object(timeout_ms)? else {
            return Err(WinDivertError::Timeout);
        };

        let packet_length = overlapped.get_result()?;

        addr_buffer.truncate((addr_len / ADDR_SIZE as u32) as usize);
        Ok((
            buffer.map(|buffer| &buffer[..packet_length as usize]),
            addr_buffer,
        ))
    }

    pub(crate) fn internal_send(&self, packet: &WinDivertPacket<L>) -> Result<u32, WinDivertError> {
        let mut injected_length = 0;

        let res = unsafe {
            BOOL(self.core.WinDivertSend(
                self.handle.0,
                packet.data.as_ptr() as *const c_void,
                packet.data.len() as u32,
                &mut injected_length,
                packet.address.as_ref(),
            ))
        }
        .ok();

        if let Err(err) = res {
            let send_error = WinDivertSendError::try_from(err)?;
            return Err(send_error.into());
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
            BOOL(self.core.WinDivertSendEx(
                self.handle.0,
                packet_buffer.as_ptr() as *const c_void,
                packet_buffer.len() as u32,
                &mut injected_length,
                0,
                address_buffer.as_ptr(),
                (ADDR_SIZE * packet_count) as u32,
                std::ptr::null_mut(),
            ))
        }
        .ok();

        if let Err(err) = res {
            let send_error = WinDivertSendError::try_from(err)?;
            return Err(send_error.into());
        }

        Ok(injected_length)
    }

    /// Methods that allows to query the driver for parameters.
    pub fn get_param(&self, param: WinDivertParam) -> Result<u64, WinDivertError> {
        let mut value = 0;
        unsafe { BOOL(WinDivertGetParam(self.handle.0, param, &mut value)) }.ok()?;
        Ok(value)
    }

    /// Method that allows setting driver parameters.
    pub fn set_param(&self, param: WinDivertParam, value: u64) -> Result<(), WinDivertError> {
        if let WinDivertParam::VersionMajor | WinDivertParam::VersionMinor = param {
            return Err(WinDivertError::Parameter(param, value));
        } else {
            Ok(unsafe { BOOL(WinDivertSetParam(self.handle.0, param, value)) }.ok()?)
        }
    }

    /// Handle close function.
    pub fn close(&mut self, action: CloseAction) -> Result<(), WinDivertError> {
        unsafe { BOOL(WinDivertClose(self.handle.0)) }.ok()?;
        match action {
            CloseAction::Uninstall => WinDivert::uninstall(),
            CloseAction::Nothing => Ok(()),
        }
    }

    /// Shutdown function.
    pub fn shutdown(&mut self, mode: WinDivertShutdownMode) -> Result<(), WinDivertError> {
        Ok(unsafe { BOOL(WinDivertShutdown(self.handle.0, mode)) }.ok()?)
    }
}

/// Utility methods for WinDivert.
impl WinDivert<()> {
    /// Maximum number of packets that can be captured/sent in a single batched operation
    pub const MAX_BATCH: u8 = windivert_sys::WINDIVERT_BATCH_MAX as u8;

    /// Method to manually install WinDivert driver.
    /// This method allow installing a sys file not located on the same folder as the dll
    pub fn install(path: &Path) -> Result<(), WinDivertError> {
        let mut mutex = InstallMutex::new()?;
        let _guard = mutex.lock()?;

        let manager = ScManager::new()?;

        if let Err(error) = WinDivertDriverService::open(&manager) {
            if let Some(ERROR_SERVICE_DOES_NOT_EXIST) = WIN32_ERROR::from_error(&error) {
                let service = WinDivertDriverService::install(&manager, path)?;
                service.register_event_source(path)?;
                service.start()?;
                service.mark_for_deletion()?;
            }
        }

        Ok(())
    }

    /// Method that tries to uninstall WinDivert driver.
    pub fn uninstall() -> Result<(), WinDivertError> {
        let manager = ScManager::new()?;
        let service = WinDivertDriverService::open(&manager)?;
        service.stop()?;
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
