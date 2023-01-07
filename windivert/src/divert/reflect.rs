use std::borrow::Cow;
use std::num::NonZeroU32;

use crate::address::WinDivertAddress;
use crate::prelude::*;

impl WinDivert<ReflectLayer> {
    /// WinDivert constructor for reflect layer.
    pub fn reflect(
        filter: impl AsRef<str>,
        priority: i16,
        flags: WinDivertFlags,
    ) -> Result<Self, WinDivertError> {
        Self::new(
            filter.as_ref(),
            WinDivertLayer::Reflect,
            priority,
            flags.set_recv_only().set_sniff(),
        )
    }

    /// Single packet blocking recv function.
    pub fn recv<'a>(
        &self,
        buffer: Option<&'a mut [u8]>,
    ) -> Result<WinDivertPacket<'a, ReflectLayer>, WinDivertError> {
        self.internal_recv(buffer)
    }

    /// Batched blocking recv function.
    pub fn recv_ex<'a>(
        &self,
        buffer: Option<&'a mut [u8]>,
        packet_count: u8,
    ) -> Result<Vec<WinDivertPacket<'a, ReflectLayer>>, WinDivertError> {
        let (mut buffer, addresses) = self.internal_recv_ex(buffer, packet_count)?;
        let mut packets = Vec::with_capacity(addresses.len());
        for addr in addresses.into_iter() {
            packets.push(WinDivertPacket {
                address: WinDivertAddress::<ReflectLayer>::from_raw(addr),
                data: buffer
                    .map(|inner_buffer| {
                        let (data, tail) = inner_buffer.split_at(
                            inner_buffer
                                .iter()
                                .position(|&x| x == b'\0')
                                .expect("CStrings always end in null"),
                        );
                        buffer = Some(tail);
                        Cow::Borrowed(data)
                    })
                    .unwrap_or_default(),
            });
        }
        Ok(packets)
    }

    /// Single packet blocking recv function with timeout.
    pub fn recv_wait<'a>(
        &self,
        buffer: Option<&'a mut [u8]>,
        timeout_ms: u32,
    ) -> Result<WinDivertPacket<'a, ReflectLayer>, WinDivertError> {
        if let Some(timeout) = NonZeroU32::new(timeout_ms) {
            self.internal_recv_wait_ex(buffer, 1, timeout)
                .map(|(data, addr)| WinDivertPacket {
                    address: WinDivertAddress::<ReflectLayer>::from_raw(addr[0]),
                    data: data.unwrap_or_default().into(),
                })
        } else {
            self.internal_recv(buffer)
        }
    }

    /// Batched blocking recv function with timeout.
    pub fn recv_wait_ex<'a>(
        &self,
        buffer: Option<&'a mut [u8]>,
        packet_count: u8,
        timeout_ms: u32,
    ) -> Result<Vec<WinDivertPacket<'a, ReflectLayer>>, WinDivertError> {
        let (mut buffer, addresses) = if let Some(timeout) = NonZeroU32::new(timeout_ms) {
            self.internal_recv_wait_ex(buffer, packet_count, timeout)?
        } else {
            self.internal_recv_ex(buffer, packet_count)?
        };
        let mut packets = Vec::with_capacity(addresses.len());
        for addr in addresses.into_iter() {
            packets.push(WinDivertPacket {
                address: WinDivertAddress::<ReflectLayer>::from_raw(addr),
                data: buffer
                    .map(|inner_buffer| {
                        let (data, tail) = inner_buffer.split_at(
                            inner_buffer
                                .iter()
                                .position(|&x| x == b'\0')
                                .expect("CStrings always end in null"),
                        );
                        buffer = Some(tail);
                        Cow::Borrowed(data)
                    })
                    .unwrap_or_default(),
            });
        }
        Ok(packets)
    }
}
