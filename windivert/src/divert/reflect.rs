use std::borrow::Cow;

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
        buffer: &'a mut [u8],
    ) -> Result<WinDivertPacket<'a, ReflectLayer>, WinDivertError> {
        self.internal_recv(Some(buffer))
    }

    /// Single packet blocking recv that won't error with [`WinDivertRecvError::InsufficientBuffer`] and will return a [partial packet](`WinDivertPartialPacket`) instead.
    pub fn partial_recv<'a>(
        &self,
        buffer: &'a mut [u8],
    ) -> Result<PacketEither<'a, ReflectLayer>, WinDivertError> {
        self.internal_partial_recv(Some(buffer))
    }

    /// Batched blocking recv function.
    pub fn recv_ex<'a>(
        &self,
        buffer: &'a mut [u8],
        packet_count: u8,
    ) -> Result<Vec<WinDivertPacket<'a, ReflectLayer>>, WinDivertError> {
        let (mut buffer, addresses) = self.internal_recv_ex(Some(buffer), packet_count)?;
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
