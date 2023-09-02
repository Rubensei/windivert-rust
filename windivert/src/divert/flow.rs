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
}
