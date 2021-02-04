// TODO: #[deny(missing_docs)]
// TODO: #![warn(missing_docs)]
/*!
Wrapper arround [`windivert_sys`] ffi crate.
*/

mod error;
mod packet;

use error::*;
use wd::{address::WINDIVERT_ADDRESS, ioctl::WINDIVERT_IOCTL_RECV};
use windivert_sys as wd;

pub use error::WinDivertError;
pub use packet::*;
pub use wd::{WinDivertFlags, WinDivertLayer, WinDivertParam, WinDivertShutdownMode};

use std::{
    convert::TryFrom,
    ffi::{c_void, CString},
    io::Result as IOResult,
    mem::MaybeUninit,
};

use etherparse::{InternetSlice, SlicedPacket};

use winapi::{
    shared::minwindef::{FALSE, TRUE},
    um::{
        handleapi::INVALID_HANDLE_VALUE,
        minwinbase::OVERLAPPED,
        winnt::HANDLE,
        winsvc::{self, SC_HANDLE, SERVICE_STATUS},
    },
};

macro_rules! try_win {
    ($expr:expr) => {
        if $expr == winapi::shared::minwindef::FALSE {
            return Err(std::io::Error::last_os_error());
        }
    };
}

macro_rules! try_divert {
    ($expr:expr) => {
        if $expr == winapi::shared::minwindef::FALSE {
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

pub struct WinDivert {
    handle: HANDLE,
    layer: WinDivertLayer,
    tls_idx: u32,
}

impl WinDivert {
    pub fn new(
        filter: String,
        layer: WinDivertLayer,
        priority: i16,
        flags: WinDivertFlags,
    ) -> Result<Self, WinDivertError> {
        let filter = CString::new(filter)?;
        let windivert_tls_idx = unsafe { winapi::um::processthreadsapi::TlsAlloc() };
        let handle =
            unsafe { wd::WinDivertOpen(filter.as_ptr(), layer.into(), priority, flags.into()) };
        if handle == INVALID_HANDLE_VALUE {
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

    fn get_event(tls_idx: u32) -> Result<winapi::um::winnt::HANDLE, WinDivertError> {
        let mut event = unsafe { winapi::um::processthreadsapi::TlsGetValue(tls_idx) };
        if event == std::ptr::null_mut() {
            event = unsafe {
                winapi::um::synchapi::CreateEventA(
                    std::ptr::null_mut(),
                    false as i32,
                    false as i32,
                    std::ptr::null_mut(),
                )
            };
            if event == std::ptr::null_mut() {
                return Err(std::io::Error::last_os_error().into());
            } else {
                unsafe { winapi::um::processthreadsapi::TlsSetValue(tls_idx, event) }
            };
        };
        Ok(event)
    }

    fn parse_packets(
        &self,
        mut buffer: Vec<u8>,
        addr_buffer: Vec<WINDIVERT_ADDRESS>,
    ) -> Vec<Packet> {
        let mut packets = Vec::new();
        for addr in addr_buffer.into_iter() {
            packets.push(Packet {
                address: addr,
                data: match self.layer {
                    WinDivertLayer::Network | WinDivertLayer::Forward => {
                        let headers = SlicedPacket::from_ip(&buffer)
                            .expect("WinDivert can't capture anything below ip");
                        let offset = match headers.ip.unwrap() {
                            InternetSlice::Ipv4(ipheader) => ipheader.total_len() as usize,
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

    pub fn recv(&self, buffer_size: usize) -> Result<Packet, WinDivertError> {
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
        } == TRUE
        {
            buffer.truncate(packet_length as usize);
            Ok(Packet {
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

    pub fn recv_ex(
        &self,
        buffer_size: usize,
        packet_count: usize,
    ) -> Result<Option<Vec<Packet>>, WinDivertError> {
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
        } == TRUE
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

    pub fn recv_wait(
        &self,
        buffer_size: usize,
        timeout_ms: u32,
    ) -> Result<Option<Packet>, WinDivertError> {
        let mut packet_length = 0u32;
        let mut buffer = vec![0u8; buffer_size];

        let mut overlapped: winapi::um::minwinbase::OVERLAPPED = unsafe { std::mem::zeroed() };
        overlapped.hEvent = WinDivert::get_event(self.tls_idx)?;

        let mut ioctl: WINDIVERT_IOCTL_RECV = unsafe { std::mem::zeroed() };
        let mut addr: WINDIVERT_ADDRESS = unsafe { std::mem::zeroed() };
        ioctl.addr = &mut addr as *mut _ as *mut c_void as u64;
        ioctl.addr_len_ptr = std::ptr::null() as *const c_void as u64;

        let res = unsafe {
            winapi::um::ioapiset::DeviceIoControl(
                self.handle as winapi::um::winnt::HANDLE,
                winapi::um::winioctl::CTL_CODE(
                    winapi::um::winioctl::FILE_DEVICE_NETWORK,
                    0x923,
                    winapi::um::winioctl::METHOD_OUT_DIRECT,
                    winapi::um::winnt::FILE_READ_DATA,
                ),
                &mut ioctl as *mut _ as *mut c_void,
                std::mem::size_of::<WINDIVERT_IOCTL_RECV>() as u32,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len() as u32,
                &mut packet_length,
                &mut overlapped,
            )
        };

        if res == FALSE
            && std::io::Error::last_os_error().raw_os_error().unwrap() as u32
                == winapi::shared::winerror::ERROR_IO_PENDING
        {
            loop {
                let res = unsafe {
                    winapi::um::ioapiset::GetOverlappedResultEx(
                        self.handle as *mut c_void,
                        &mut overlapped,
                        &mut packet_length,
                        timeout_ms,
                        true as i32,
                    )
                };
                if res == TRUE {
                    break;
                } else {
                    match std::io::Error::last_os_error().raw_os_error().unwrap() as u32 {
                        winapi::shared::winerror::WAIT_TIMEOUT => {
                            unsafe { winapi::um::ioapiset::CancelIo(self.handle as *mut c_void) };
                            return Ok(None);
                        }
                        winapi::um::winbase::WAIT_IO_COMPLETION => break,
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
        Ok(Some(Packet {
            address: addr,
            data: buffer,
        }))
    }

    pub fn recv_ex_wait(
        &self,
        buffer_size: usize,
        timeout_ms: u32,
        packet_count: usize,
    ) -> Result<Option<Vec<Packet>>, WinDivertError> {
        let mut packet_length = 0u32;
        let mut buffer = vec![0u8; buffer_size * packet_count];

        let mut overlapped: winapi::um::minwinbase::OVERLAPPED = unsafe { std::mem::zeroed() };
        overlapped.hEvent = WinDivert::get_event(self.tls_idx)?;

        let mut ioctl: WINDIVERT_IOCTL_RECV = unsafe { std::mem::zeroed() };
        let mut addr_buffer: Vec<WINDIVERT_ADDRESS> =
            vec![WINDIVERT_ADDRESS::default(); packet_count];
        let mut addr_len = (std::mem::size_of::<WINDIVERT_ADDRESS>() * packet_count) as u32;
        ioctl.addr = &mut addr_buffer[0] as *mut _ as u64;
        ioctl.addr_len_ptr = &mut addr_len as *mut u32 as u64;
        let res = unsafe {
            winapi::um::ioapiset::DeviceIoControl(
                self.handle as winapi::um::winnt::HANDLE,
                winapi::um::winioctl::CTL_CODE(
                    winapi::um::winioctl::FILE_DEVICE_NETWORK,
                    0x923,
                    winapi::um::winioctl::METHOD_OUT_DIRECT,
                    winapi::um::winnt::FILE_READ_DATA,
                ),
                &mut ioctl as *mut _ as *mut c_void,
                std::mem::size_of::<WINDIVERT_IOCTL_RECV>() as u32,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len() as u32,
                &mut packet_length,
                &mut overlapped,
            )
        };

        if res == FALSE
            && std::io::Error::last_os_error().raw_os_error().unwrap() as u32
                == winapi::shared::winerror::ERROR_IO_PENDING
        {
            loop {
                let res = unsafe {
                    winapi::um::ioapiset::GetOverlappedResultEx(
                        self.handle as *mut c_void,
                        &mut overlapped,
                        &mut packet_length,
                        timeout_ms,
                        TRUE,
                    )
                };

                if res == TRUE {
                    break;
                } else {
                    match std::io::Error::last_os_error().raw_os_error().unwrap() as u32 {
                        winapi::shared::winerror::WAIT_TIMEOUT => {
                            unsafe { winapi::um::ioapiset::CancelIo(self.handle as *mut c_void) };
                            return Ok(None);
                        }
                        winapi::um::winbase::WAIT_IO_COMPLETION => break,
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

    pub fn send(&self, mut packet: Packet) -> Result<u32, WinDivertError> {
        let mut injected_length = 0;
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

    pub fn send_ex(&self, mut data: Vec<Packet>) -> Result<u32, WinDivertError> {
        let packet_count = data.len();
        let mut injected_length = 0;
        let mut packet_buffer = Vec::new();
        let mut address_buffer: Vec<WINDIVERT_ADDRESS> = Vec::new();
        data.drain(..).for_each(|mut packet| {
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

    pub fn close(&mut self, action: CloseAction) -> IOResult<()> {
        unsafe { try_win!(wd::WinDivertClose(self.handle)) };
        match action {
            CloseAction::Uninstall => WinDivert::uninstall(),
            CloseAction::Nothing => Ok(()),
        }
    }

    pub fn get_param(&self, param: WinDivertParam) -> Result<u64, WinDivertError> {
        let mut value = 0;
        unsafe {
            try_divert!(wd::WinDivertGetParam(self.handle, param.into(), &mut value));
        }
        Ok(value)
    }

    pub fn set_param(&self, param: WinDivertParam, value: u64) -> Result<(), WinDivertError> {
        unsafe { try_divert!(wd::WinDivertSetParam(self.handle, param.into(), value)) }
        Ok(())
    }

    pub fn shutdown(&mut self, mode: WinDivertShutdownMode) -> IOResult<()> {
        unsafe { try_win!(wd::WinDivertShutdown(self.handle, mode.into())) };
        Ok(())
    }

    pub fn uninstall() -> IOResult<()> {
        let service_name = std::ffi::CString::new("WinDivert").unwrap();
        let status: *mut SERVICE_STATUS = MaybeUninit::uninit().as_mut_ptr();
        unsafe {
            let manager: SC_HANDLE = winsvc::OpenSCManagerA(
                std::ptr::null(),
                std::ptr::null(),
                winsvc::SC_MANAGER_ALL_ACCESS,
            );
            if manager == std::ptr::null_mut() {
                return Err(std::io::Error::last_os_error());
            }
            let service: SC_HANDLE =
                winsvc::OpenServiceA(manager, service_name.as_ptr(), winsvc::SERVICE_ALL_ACCESS);
            if service == std::ptr::null_mut() {
                return Err(std::io::Error::last_os_error());
            }
            try_win!(winsvc::ControlService(
                service,
                winsvc::SERVICE_CONTROL_STOP,
                status
            ));
            try_win!(winsvc::CloseServiceHandle(service));
            try_win!(winsvc::CloseServiceHandle(manager));
        }
        Ok(())
    }
}
