// TODO: #[deny(missing_docs)]
#![warn(missing_docs)]
/*!
Wrapper arround [`windivert_sys`] ffi crate.
*/

/// WinDivert address data structures
pub mod address;
mod error;
mod packet;

use error::*;
use wd::{address::WINDIVERT_ADDRESS, ioctl::WINDIVERT_IOCTL_RECV};
use windivert_sys as wd;
use windows::{
    core::{Error as WinError, Result as WinResult, HRESULT},
    Devices::Custom::{IOControlAccessMode, IOControlBufferingMethod, IOControlCode},
    Win32::{
        Foundation::{BOOL, ERROR_IO_PENDING, HANDLE, PSTR, WAIT_TIMEOUT},
        Security::SC_HANDLE,
        System::{
            Ioctl::FILE_DEVICE_NETWORK,
            Services::{
                CloseServiceHandle, ControlService, OpenSCManagerA, OpenServiceA,
                SC_MANAGER_ALL_ACCESS, SERVICE_CONTROL_STOP, SERVICE_STATUS,
            },
            Threading::{CreateEventA, TlsAlloc, TlsGetValue, TlsSetValue, WAIT_IO_COMPLETION},
            IO::{CancelIo, DeviceIoControl, GetOverlappedResultEx, OVERLAPPED},
        },
    },
};

pub use error::WinDivertError;
pub use packet::*;
pub use wd::{
    WinDivertEvent, WinDivertFlags, WinDivertLayer, WinDivertParam, WinDivertShutdownMode,
};

use std::{
    convert::TryFrom,
    ffi::{c_void, CString},
    mem::MaybeUninit,
};

use etherparse::{InternetSlice, SlicedPacket};

macro_rules! try_win {
    ($expr:expr) => {{
        let x = $expr;
        if x == BOOL::from(false) {
            return Err(WinError::fast_error(HRESULT(
                std::io::Error::last_os_error().raw_os_error().unwrap(),
            )));
        } else {
            x
        }
    }};

    ($expr:expr, $value:expr) => {{
        let x = $expr;
        if x == $value {
            return Err(WinError::fast_error(HRESULT(
                std::io::Error::last_os_error().raw_os_error().unwrap(),
            )));
        } else {
            x
        }
    }};
}

macro_rules! try_divert {
    ($expr:expr) => {
        if $expr == BOOL::from(false) {
            return Err(std::io::Error::last_os_error().into());
        }
    };
}

const ADDR_SIZE: usize = std::mem::size_of::<WINDIVERT_ADDRESS>();

/// Action parameter for  [`WinDivert::close()`](`fn@WinDivert::close`)
pub enum CloseAction {
    /// Close the handle and try to uninstall the WinDivert driver.
    Uninstall,
    /// Close the handle without uninstalling the driver.
    Nothing,
}

/// Main wrapper struct around windivert functionalities.
pub struct WinDivert {
    handle: HANDLE,
    layer: WinDivertLayer,
    tls_idx: u32,
}

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
            unsafe { wd::WinDivertOpen(filter.as_ptr(), layer.into(), priority, flags.into()) };
        if handle.is_invalid() {
            match WinDivertOpenError::try_from(std::io::Error::last_os_error()) {
                Ok(err) => Err(WinDivertError::Open(err)),
                Err(err) => Err(WinDivertError::OSError(err)),
            }
        } else {
            Ok(Self {
                handle: handle,
                layer,
                tls_idx: windivert_tls_idx,
            })
        }
    }

    fn get_event(tls_idx: u32) -> Result<HANDLE, WinDivertError> {
        let mut event = HANDLE::default();
        event.0 = unsafe { TlsGetValue(tls_idx) } as isize;
        if event.is_invalid() {
            let event =
                unsafe { CreateEventA(std::ptr::null_mut(), false, false, PSTR::default()) };
            if event.is_invalid() {
                return Err(std::io::Error::last_os_error().into());
            } else {
                unsafe { TlsSetValue(tls_idx, event.0 as *mut c_void) }
            };
        };
        Ok(event)
    }

    fn parse_packets(
        &self,
        mut buffer: Vec<u8>,
        addr_buffer: Vec<WINDIVERT_ADDRESS>,
    ) -> Vec<WinDivertPacket> {
        let mut packets = Vec::with_capacity(addr_buffer.len());
        for addr in addr_buffer.into_iter() {
            packets.push(WinDivertPacket {
                address: addr,
                data: match self.layer {
                    WinDivertLayer::Network | WinDivertLayer::Forward => {
                        let headers = SlicedPacket::from_ip(&buffer)
                            .expect("WinDivert can't capture anything below ip");
                        let offset = match headers.ip.unwrap() {
                            InternetSlice::Ipv4(ipheader, _) => ipheader.total_len() as usize,
                            InternetSlice::Ipv6(ip6header, _) => {
                                ip6header.payload_length() as usize + 40
                            }
                        };
                        let aux = buffer.split_off(offset);
                        let data = buffer;
                        buffer = aux;
                        data
                    }
                    WinDivertLayer::Reflect => {
                        let aux = buffer.split_off(
                            buffer
                                .iter()
                                .position(|&x| x == b'\0')
                                .expect("CStrings always end in null"),
                        );
                        let data = buffer;
                        buffer = aux;
                        data
                    }
                    _ => Vec::new(),
                },
            });
        }
        packets
    }

    /// Single packet blocking recv function.
    pub fn recv(&self, buffer_size: usize) -> Result<WinDivertPacket, WinDivertError> {
        let mut packet_length = 0;
        let mut buffer = vec![0u8; buffer_size];
        let mut addr = WINDIVERT_ADDRESS::default();
        if unsafe {
            wd::WinDivertRecv(
                self.handle,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len() as u32,
                &mut packet_length,
                &mut addr,
            )
        }
        .as_bool()
        {
            buffer.truncate(packet_length as usize);
            Ok(WinDivertPacket {
                address: addr,
                data: buffer,
            })
        } else {
            let err = WinDivertRecvError::try_from(std::io::Error::last_os_error());
            match err {
                Ok(err) => Err(WinDivertError::Recv(err)),
                Err(err) => Err(WinDivertError::OSError(err)),
            }
        }
    }

    /// Batched blocking recv function.
    pub fn recv_ex(
        &self,
        buffer_size: usize,
        packet_count: usize,
    ) -> Result<Option<Vec<WinDivertPacket>>, WinDivertError> {
        let mut packet_length = 0;
        let mut buffer = vec![0u8; buffer_size];

        let mut addr_len = (ADDR_SIZE * packet_count) as u32;
        let mut addr_buffer: Vec<WINDIVERT_ADDRESS> =
            vec![WINDIVERT_ADDRESS::default(); packet_count];

        if unsafe {
            wd::WinDivertRecvEx(
                self.handle,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len() as u32,
                &mut packet_length,
                0,
                addr_buffer.as_mut_ptr(),
                &mut addr_len,
                std::ptr::null_mut() as *mut OVERLAPPED,
            )
        }
        .as_bool()
        {
            addr_buffer.truncate((addr_len / ADDR_SIZE as u32) as usize);
            buffer.truncate(packet_length as usize);
            Ok(Some(self.parse_packets(buffer, addr_buffer)))
        } else {
            let err = WinDivertRecvError::try_from(std::io::Error::last_os_error());
            match err {
                Ok(err) => Err(WinDivertError::Recv(err)),
                Err(err) => Err(WinDivertError::OSError(err)),
            }
        }
    }

    /// Single packet recv with timout.
    pub fn recv_wait(
        &self,
        buffer_size: usize,
        timeout_ms: u32,
    ) -> Result<Option<WinDivertPacket>, WinDivertError> {
        let mut packet_length = 0u32;
        let mut buffer = vec![0u8; buffer_size];

        let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };
        overlapped.hEvent = WinDivert::get_event(self.tls_idx)?;

        let mut ioctl: WINDIVERT_IOCTL_RECV = unsafe { std::mem::zeroed() };
        let mut addr: WINDIVERT_ADDRESS = unsafe { std::mem::zeroed() };
        ioctl.addr = &mut addr as *mut _ as *mut c_void as u64;
        ioctl.addr_len_ptr = std::ptr::null() as *const c_void as u64;

        let res = unsafe {
            DeviceIoControl(
                self.handle,
                IOControlCode::CreateIOControlCode(
                    FILE_DEVICE_NETWORK as u16,
                    0x923,
                    IOControlAccessMode::Read,
                    IOControlBufferingMethod::DirectOutput,
                )
                .unwrap()
                .ControlCode()
                .unwrap(),
                &mut ioctl as *mut _ as *mut c_void,
                std::mem::size_of::<WINDIVERT_IOCTL_RECV>() as u32,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len() as u32,
                &mut packet_length,
                &mut overlapped,
            )
        };

        if !res.as_bool()
            && std::io::Error::last_os_error().raw_os_error().unwrap() as u32 == ERROR_IO_PENDING
        {
            loop {
                let res = unsafe {
                    GetOverlappedResultEx(
                        self.handle,
                        &mut overlapped,
                        &mut packet_length,
                        timeout_ms,
                        true,
                    )
                };
                if res.as_bool() {
                    break;
                } else {
                    let return_cause =
                        std::io::Error::last_os_error().raw_os_error().unwrap() as u32;
                    match return_cause {
                        WAIT_TIMEOUT => {
                            unsafe { CancelIo(self.handle) };
                            return Ok(None);
                        }
                        WAIT_IO_COMPLETION => break,
                        value => {
                            if let Ok(err) = WinDivertRecvError::try_from(value as i32) {
                                return Err(WinDivertError::Recv(err));
                            } else {
                                panic!("This arm should never be reached")
                            }
                        }
                    }
                }
            }
        }
        buffer.truncate(packet_length as usize);
        Ok(Some(WinDivertPacket::from(WinDivertPacket {
            address: addr,
            data: buffer,
        })))
    }

    /// Bacthed recv function with timeout.
    pub fn recv_ex_wait(
        &self,
        buffer_size: usize,
        timeout_ms: u32,
        packet_count: usize,
    ) -> Result<Option<Vec<WinDivertPacket>>, WinDivertError> {
        let mut packet_length = 0u32;
        let mut buffer = vec![0u8; buffer_size * packet_count];

        let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };
        overlapped.hEvent = WinDivert::get_event(self.tls_idx)?;

        let mut ioctl: WINDIVERT_IOCTL_RECV = unsafe { std::mem::zeroed() };
        let mut addr_buffer: Vec<WINDIVERT_ADDRESS> =
            vec![WINDIVERT_ADDRESS::default(); packet_count];
        let mut addr_len = (std::mem::size_of::<WINDIVERT_ADDRESS>() * packet_count) as u32;
        ioctl.addr = &mut addr_buffer[0] as *mut _ as u64;
        ioctl.addr_len_ptr = &mut addr_len as *mut u32 as u64;
        let res = unsafe {
            DeviceIoControl(
                self.handle,
                IOControlCode::CreateIOControlCode(
                    FILE_DEVICE_NETWORK as u16,
                    0x923,
                    IOControlAccessMode::Read,
                    IOControlBufferingMethod::DirectOutput,
                )
                .unwrap()
                .ControlCode()
                .unwrap(),
                &mut ioctl as *mut _ as *mut c_void,
                std::mem::size_of::<WINDIVERT_IOCTL_RECV>() as u32,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len() as u32,
                &mut packet_length,
                &mut overlapped,
            )
        };

        if !res.as_bool()
            && std::io::Error::last_os_error().raw_os_error().unwrap() as u32 == ERROR_IO_PENDING
        {
            loop {
                let res = unsafe {
                    GetOverlappedResultEx(
                        self.handle,
                        &mut overlapped,
                        &mut packet_length,
                        timeout_ms,
                        true,
                    )
                };

                if res.as_bool() {
                    break;
                } else {
                    let return_cause =
                        std::io::Error::last_os_error().raw_os_error().unwrap() as u32;
                    match return_cause {
                        WAIT_TIMEOUT => {
                            unsafe { CancelIo(self.handle) };
                            return Ok(None);
                        }
                        WAIT_IO_COMPLETION => break,
                        value => {
                            if let Ok(err) = WinDivertRecvError::try_from(value as i32) {
                                return Err(WinDivertError::Recv(err));
                            } else {
                                panic!("This arm should never be reached")
                            }
                        }
                    }
                }
            }
        }
        addr_buffer.truncate((addr_len / ADDR_SIZE as u32) as usize);
        buffer.truncate(packet_length as usize);
        Ok(Some(self.parse_packets(buffer, addr_buffer)))
    }

    /// Single packet send function.
    pub fn send<T: Into<WinDivertPacket>>(&self, packet: T) -> Result<u32, WinDivertError> {
        let mut injected_length = 0;
        let mut packet = packet.into();
        unsafe {
            try_divert!(wd::WinDivertSend(
                self.handle,
                packet.data.as_mut_ptr() as *const c_void,
                packet.data.len() as u32,
                &mut injected_length,
                &packet.address,
            ))
        }
        Ok(injected_length)
    }

    /// Batched send function.
    pub fn send_ex<T: Into<WinDivertPacket>>(
        &self,
        mut data: Vec<T>,
    ) -> Result<u32, WinDivertError> {
        let packet_count = data.len();
        let mut injected_length = 0;
        let mut packet_buffer = Vec::with_capacity(data.len());
        let mut address_buffer: Vec<WINDIVERT_ADDRESS> = Vec::with_capacity(data.len());
        data.drain(..).for_each(|packet| {
            let mut packet: WinDivertPacket = packet.into();
            packet_buffer.append(&mut packet.data);
            address_buffer.push(packet.address);
        });
        unsafe {
            try_divert!(wd::WinDivertSendEx(
                self.handle,
                packet_buffer.as_mut_ptr() as *const c_void,
                packet_buffer.len() as u32,
                &mut injected_length,
                0,
                address_buffer.as_ptr(),
                (std::mem::size_of::<WINDIVERT_ADDRESS>() * packet_count) as u32,
                std::ptr::null_mut(),
            ))
        };
        Ok(injected_length)
    }

    /// Handle close function.
    pub fn close(&mut self, action: CloseAction) -> WinResult<()> {
        unsafe { try_win!(wd::WinDivertClose(self.handle)) };
        match action {
            CloseAction::Uninstall => WinDivert::uninstall(),
            CloseAction::Nothing => Ok(()),
        }
    }

    /// Methods that allows to query the driver for parameters.
    pub fn get_param(&self, param: WinDivertParam) -> Result<u64, WinDivertError> {
        let mut value = 0;
        unsafe {
            try_divert!(wd::WinDivertGetParam(self.handle, param.into(), &mut value));
        }
        Ok(value)
    }

    /// Method that allows setting driver parameters.
    pub fn set_param(&self, param: WinDivertParam, value: u64) -> Result<(), WinDivertError> {
        match param {
            WinDivertParam::VersionMajor | WinDivertParam::VersionMinor => {
                Err(WinDivertError::Parameter)
            }
            _ => {
                unsafe { try_divert!(wd::WinDivertSetParam(self.handle, param.into(), value)) }
                Ok(())
            }
        }
    }

    /// Shutdown function.
    pub fn shutdown(&mut self, mode: WinDivertShutdownMode) -> WinResult<()> {
        unsafe { try_win!(wd::WinDivertShutdown(self.handle, mode.into())) };
        Ok(())
    }

    /// Method that tries to uninstall WinDivert driver.
    pub fn uninstall() -> WinResult<()> {
        let status: *mut SERVICE_STATUS = MaybeUninit::uninit().as_mut_ptr();
        unsafe {
            let manager = try_win!(
                OpenSCManagerA(PSTR::default(), PSTR::default(), SC_MANAGER_ALL_ACCESS),
                SC_HANDLE::default()
            );
            let service = try_win!(
                OpenServiceA(
                    manager,
                    PSTR(String::from("WinDivert").as_mut_ptr()),
                    SC_MANAGER_ALL_ACCESS
                ),
                SC_HANDLE::default()
            );
            try_win!(ControlService(service, SERVICE_CONTROL_STOP, status));
            try_win!(CloseServiceHandle(service));
            try_win!(CloseServiceHandle(manager));
        }
        Ok(())
    }
}
