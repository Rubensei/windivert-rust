// TODO: #[deny(missing_docs)]
#![warn(missing_docs)]
/*!
Wrapper around [`windivert_sys`] ffi crate.
*/

/// WinDivert address data structures
pub mod address;
mod divert;
mod error;
mod layer;
mod packet;

/// Prelude module for [`WinDivert`].
pub mod prelude {
    pub use windivert_sys::{
        WinDivertEvent, WinDivertFlags, WinDivertLayer, WinDivertParam, WinDivertShutdownMode,
    };

    pub use crate::divert::*;
    pub use crate::error::*;
    pub use crate::packet::*;
}

/*
impl WinDivert {
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
                            InternetSlice::Ipv4(ip_header, _) => ip_header.total_len() as usize,
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
            sys::WinDivertRecvEx(
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
                Some(&mut ioctl as *mut _ as *mut c_void),
                std::mem::size_of::<WINDIVERT_IOCTL_RECV>() as u32,
                Some(buffer.as_mut_ptr() as *mut c_void),
                buffer.len() as u32,
                Some(&mut packet_length),
                Some(&mut overlapped),
            )
        };

        if !res.as_bool() && unsafe { GetLastError() } == ERROR_IO_PENDING {
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
                    match unsafe { GetLastError() } {
                        WAIT_TIMEOUT => {
                            unsafe { CancelIo(self.handle) };
                            return Ok(None);
                        }
                        WAIT_IO_COMPLETION => break,
                        value => {
                            if let Ok(error) = WinDivertRecvError::try_from(value.0 as i32) {
                                return Err(WinDivertError::Recv(error));
                            } else {
                                panic!("This arm should never be reached")
                            }
                        }
                    }
                }
            }
        }
        buffer.truncate(packet_length as usize);
        Ok(Some(WinDivertPacket {
            address: addr,
            data: buffer,
        }))
    }

    /// Batched recv function with timeout.
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
                Some(&mut ioctl as *mut _ as *mut c_void),
                std::mem::size_of::<WINDIVERT_IOCTL_RECV>() as u32,
                Some(buffer.as_mut_ptr() as *mut c_void),
                buffer.len() as u32,
                Some(&mut packet_length),
                Some(&mut overlapped),
            )
        };

        if !res.as_bool() && unsafe { GetLastError() } == ERROR_IO_PENDING {
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
                    match unsafe { GetLastError() } {
                        WAIT_TIMEOUT => {
                            unsafe { CancelIo(self.handle) };
                            return Ok(None);
                        }
                        WAIT_IO_COMPLETION => break,
                        value => {
                            if let Ok(error) = WinDivertRecvError::try_from(value.0 as i32) {
                                return Err(WinDivertError::Recv(error));
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
}
*/
