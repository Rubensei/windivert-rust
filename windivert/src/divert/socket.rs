use crate::address::WinDivertAddress;
use crate::prelude::*;

impl WinDivert<SocketLayer> {
    /// WinDivert constructor for socket layer.
    pub fn socket(
        filter: impl AsRef<str>,
        priority: i16,
        flags: WinDivertFlags,
    ) -> Result<Self, WinDivertError> {
        Self::new(
            filter.as_ref(),
            WinDivertLayer::Socket,
            priority,
            flags.set_recv_only(),
        )
    }

    /// Single packet blocking recv function.
    pub fn recv<'a>(&self) -> Result<WinDivertPacket<'a, SocketLayer>, WinDivertError> {
        self.internal_recv(None)
    }

    /// Batched blocking recv function.
    pub fn recv_ex<'a>(
        &self,
        packet_count: u8,
    ) -> Result<Vec<WinDivertPacket<'a, SocketLayer>>, WinDivertError> {
        let (_, addresses) = self.internal_recv_ex(None, packet_count)?;
        let mut packets = Vec::with_capacity(addresses.len());
        for addr in addresses.into_iter() {
            packets.push(WinDivertPacket::<SocketLayer> {
                address: WinDivertAddress::<SocketLayer>::from_raw(addr),
                data: Default::default(),
            });
        }
        Ok(packets)
    }
}
