use super::address::*;
use std::fmt::Debug;

use windivert_sys::address::WINDIVERT_ADDRESS;

#[derive(Debug)]
/// Raw packet
pub struct WinDivertPacket {
    pub(crate) address: WINDIVERT_ADDRESS,
    /// Raw captured data
    pub data: Vec<u8>,
}

impl WinDivertPacket {
    /// Parse a raw packet
    pub fn parse(self) -> WinDivertParsedPacket {
        match self.address.layer() {
            windivert_sys::WinDivertLayer::Network | windivert_sys::WinDivertLayer::Forward => {
                WinDivertParsedPacket::Network {
                    addr: WinDivertNetworkData { data: self.address },
                    data: self.data,
                }
            }
            windivert_sys::WinDivertLayer::Flow => WinDivertParsedPacket::Flow {
                addr: WinDivertFlowData { data: self.address },
            },
            windivert_sys::WinDivertLayer::Socket => WinDivertParsedPacket::Socket {
                addr: WinDivertSocketData { data: self.address },
            },
            windivert_sys::WinDivertLayer::Reflect => WinDivertParsedPacket::Reflect {
                addr: WinDivertReflectData { data: self.address },
                filter: self.data,
            },
        }
    }
}

#[derive(Debug)]
/// Parsed packet type.
pub enum WinDivertParsedPacket {
    /// Packet type returned by handles using [`WinDivertLayer::Network`](super::WinDivertLayer::Network).
    Network {
        /// WinDivert data associated with the packet.
        addr: WinDivertNetworkData,
        /// Raw captured data.
        data: Vec<u8>,
    },
    /// Packet type returned by handles using [`WinDivertLayer::Flow`](super::WinDivertLayer::Flow).
    Flow {
        /// WinDivert data associated with the packet.
        addr: WinDivertFlowData,
    },
    /// Packet type returned by handles using [`WinDivertLayer::Socket`](super::WinDivertLayer::Socket).
    Socket {
        /// WinDivert data associated with the packet.
        addr: WinDivertSocketData,
    },
    /// Packet type returned by handles using [`WinDivertLayer::Reflect`](super::WinDivertLayer::Reflect).
    Reflect {
        /// WinDivert data associated with the packet.
        addr: WinDivertReflectData,
        /// Object string representation of the filter used to open the handle.
        filter: Vec<u8>,
    },
}

impl From<WinDivertParsedPacket> for WinDivertPacket {
    fn from(packet: WinDivertParsedPacket) -> Self {
        let (buffer, addr) = match packet {
            WinDivertParsedPacket::Network { addr, data } => (data, addr.data),
            WinDivertParsedPacket::Flow { addr } => (Vec::new(), addr.data),
            WinDivertParsedPacket::Socket { addr } => (Vec::new(), addr.data),
            WinDivertParsedPacket::Reflect { addr, filter } => (filter, addr.data),
        };
        WinDivertPacket {
            address: addr,
            data: buffer,
        }
    }
}
