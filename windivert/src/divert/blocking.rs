use std::{ffi::c_void, mem::MaybeUninit};

use crate::address::WinDivertAddress;
use crate::layer;
use crate::prelude::*;
use sys::address::WINDIVERT_ADDRESS;
use windivert_sys as sys;

// TODO: Batch recv

impl<L: layer::WinDivertLayerTrait> WinDivert<L> {
    /// Single packet blocking recv function.
    pub fn recv<'a>(
        &self,
        buffer: Option<&'a mut [u8]>,
    ) -> Result<Option<WinDivertPacket<'a, L>>, WinDivertError> {
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
            Ok(buffer.map(|buffer| WinDivertPacket {
                address: WinDivertAddress::<L>::from_raw(unsafe { addr.assume_init() }),
                data: buffer[..packet_length as usize].into(),
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
    pub fn send(&self, packet: &WinDivertPacket<L>) -> Result<u32, WinDivertError> {
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

    /// Batched send function.
    pub fn send_ex<'data, 'packets, P, I>(&self, packets: P) -> Result<u32, WinDivertError>
    where
        P: IntoIterator<IntoIter = I>,
        I: ExactSizeIterator<Item = &'packets WinDivertPacket<'data, L>>,
        'data: 'packets,
        L: 'packets,
    {
        let mut packets = packets.into_iter();
        let packet_count = packets.len();
        let mut injected_length = 0;
        let capacity = packets.by_ref().map(|p| p.data.len()).sum();
        let mut packet_buffer: Vec<u8> = Vec::with_capacity(capacity);
        let mut address_buffer: Vec<WINDIVERT_ADDRESS> = Vec::with_capacity(packet_count);
        packets.for_each(|packet: &'packets WinDivertPacket<'data, L>| {
            packet_buffer.extend(&packet.data[..]);
            address_buffer.push(packet.address.as_ref().clone());
        });

        let res = unsafe {
            sys::WinDivertSendEx(
                self.handle,
                packet_buffer.as_ptr() as *const c_void,
                packet_buffer.len() as u32,
                &mut injected_length,
                0,
                address_buffer.as_ptr(),
                (std::mem::size_of::<WINDIVERT_ADDRESS>() * packet_count) as u32,
                std::ptr::null_mut(),
            )
        };

        if !res.as_bool() {
            return Err(std::io::Error::last_os_error().into());
        }

        Ok(injected_length)
    }
}

impl WinDivert<layer::NetworkLayer> {}

impl WinDivert<layer::ForwardLayer> {}

impl WinDivert<layer::FlowLayer> {}

impl WinDivert<layer::SocketLayer> {}

impl WinDivert<layer::ReflectLayer> {}
