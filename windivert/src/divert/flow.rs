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
    pub fn recv<'a>(&self) -> Result<WinDivertPacket<'a, FlowLayer>, WinDivertError> {
        self.internal_recv(None)
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
    /// A timeout of 0 will return the queued data without blocking
    pub fn recv_wait<'a>(
        &self,
        timeout_ms: u32,
    ) -> Result<Option<WinDivertPacket<'a, FlowLayer>>, WinDivertError> {
        self.internal_recv_wait_ex(None, 1, timeout_ms)
            .map(|result| {
                let Some((data, addr)) = result else {
                    return None;
                };
                Some(WinDivertPacket {
                    address: WinDivertAddress::<FlowLayer>::from_raw(addr[0]),
                    data: data.unwrap_or_default().into(),
                })
            })
    }

    /// Batched blocking recv function with timeout.
    /// A timeout of 0 will return the queued data without blocking
    pub fn recv_wait_ex<'a>(
        &self,
        packet_count: u8,
        timeout_ms: u32,
    ) -> Result<Vec<WinDivertPacket<'a, FlowLayer>>, WinDivertError> {
        let Some((_, addresses)) = self.internal_recv_wait_ex(None, packet_count, timeout_ms)?
        else {
            return Ok(Vec::new());
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
