use crate::{
    error::WinDivertRecvError, WinDivert, WinDivertError, WinDivertPacket, WinDivertPacketSlice,
};
use std::{ffi::c_void, mem::MaybeUninit};
use windivert_sys as sys;

impl WinDivert {
    /// Single packet blocking recv function.
    pub fn recv<'a>(
        &self,
        buffer: Option<&'a mut [u8]>,
    ) -> Result<Option<WinDivertPacketSlice<'a>>, WinDivertError> {
        let mut packet_length = 0;
        let mut addr = MaybeUninit::uninit();
        let (buffer_ptr, buffer_len) = if let Some(buffer) = &buffer {
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
            Ok(buffer.map(|buffer| WinDivertPacketSlice {
                address: unsafe { addr.assume_init() },
                data: &mut buffer[..packet_length as usize],
            }))
        } else {
            let err = WinDivertRecvError::try_from(std::io::Error::last_os_error());
            match err {
                Ok(err) => Err(WinDivertError::Recv(err)),
                Err(err) => Err(WinDivertError::OSError(err)),
            }
        }
    }

    /// Single packet send function.
    pub fn send<T: Into<WinDivertPacket>>(&self, packet: T) -> Result<u32, WinDivertError> {
        let mut injected_length = 0;
        let mut packet = packet.into();

        let res = unsafe {
            sys::WinDivertSend(
                self.handle,
                packet.data.as_mut_ptr() as *const c_void,
                packet.data.len() as u32,
                &mut injected_length,
                &packet.address,
            )
        };

        if !res.as_bool() {
            return Err(std::io::Error::last_os_error().into());
        }

        Ok(injected_length)
    }
}
