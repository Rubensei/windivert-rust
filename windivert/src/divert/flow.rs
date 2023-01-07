use std::num::NonZeroU32;

use crate::address::WinDivertAddress;
use crate::prelude::*;

impl WinDivert<FlowLayer> {
    /// WinDivert constructor for flow layer.
    pub fn flow(
        filter: &str,
        priority: i16,
        flags: WinDivertFlags,
    ) -> Result<Self, WinDivertError> {
        Self::new(
            filter,
            WinDivertLayer::Flow,
            priority,
            flags.set_recv_only().set_sniff(),
        )
    }

    /// Single packet blocking recv function.
    pub fn recv<'a>(
        &self,
        buffer: Option<&'a mut [u8]>,
    ) -> Result<WinDivertPacket<'a, FlowLayer>, WinDivertError> {
        self.internal_recv(buffer)
    }

    /// Batched blocking recv function.
    pub fn recv_ex<'a>(
        &self,
        packet_count: u8,
    ) -> Result<Vec<WinDivertPacket<'a, FlowLayer>>, WinDivertError> {
        let (_, addresses) = self.internal_recv_ex(None, packet_count)?;
        let mut packets = Vec::with_capacity(addresses.len());
        for addr in addresses.into_iter() {
            packets.push(WinDivertPacket::<FlowLayer> {
                address: WinDivertAddress::<FlowLayer>::from_raw(addr),
                data: Default::default(),
            });
        }
        Ok(packets)
    }
    /// Single packet blocking recv function with timeout.
    pub fn recv_wait<'a>(
        &self,
        buffer: Option<&'a mut [u8]>,
        timeout_ms: u32,
    ) -> Result<WinDivertPacket<'a, FlowLayer>, WinDivertError> {
        if let Some(timeout) = NonZeroU32::new(timeout_ms) {
            self.internal_recv_wait_ex(buffer, 1, timeout)
                .map(|(data, addr)| WinDivertPacket {
                    address: WinDivertAddress::<FlowLayer>::from_raw(addr[0]),
                    data: data.unwrap_or_default().into(),
                })
        } else {
            self.internal_recv(buffer)
        }
    }

    /// Batched blocking recv function with timeout.
    pub fn recv_wait_ex<'a>(
        &self,
        packet_count: u8,
        timeout_ms: u32,
    ) -> Result<Vec<WinDivertPacket<'a, FlowLayer>>, WinDivertError> {
        let (_, addresses) = if let Some(timeout) = NonZeroU32::new(timeout_ms) {
            self.internal_recv_wait_ex(None, packet_count, timeout)?
        } else {
            self.internal_recv_ex(None, packet_count)?
        };
        let mut packets = Vec::with_capacity(addresses.len());
        for addr in addresses.into_iter() {
            packets.push(WinDivertPacket::<FlowLayer> {
                address: WinDivertAddress::<FlowLayer>::from_raw(addr),
                data: Default::default(),
            });
        }
        Ok(packets)
    }
}
