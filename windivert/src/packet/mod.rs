mod full;
mod partial;

pub use full::WinDivertPacket;
pub use partial::WinDivertPartialPacket;

use crate::layer;

/// Either a full or partial packet
pub enum PacketEither<'a, L: layer::WinDivertLayerTrait> {
    /// Full packet
    Full(WinDivertPacket<'a, L>),
    /// Partial packet
    Partial(WinDivertPartialPacket<'a, L>),
}

impl<'a, L: layer::WinDivertLayerTrait> PacketEither<'a, L> {
    /// Treat this packet as a partial packet
    pub fn to_partial(self) -> WinDivertPartialPacket<'a, L> {
        match self {
            PacketEither::Full(packet) => WinDivertPartialPacket {
                address: packet.address,
                data: packet.data,
            },
            PacketEither::Partial(packet) => packet,
        }
    }

    /// Treat this packet as a full packet
    /// # Safety
    /// This is unsafe because the packet may not be a full packet.
    /// A partial packet cant be sent/injected.
    pub unsafe fn to_full(self) -> WinDivertPacket<'a, L> {
        match self {
            PacketEither::Full(packet) => packet,
            PacketEither::Partial(packet) => WinDivertPacket {
                address: packet.address,
                data: packet.data,
            },
        }
    }
}
