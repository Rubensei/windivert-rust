use super::address::*;
use std::fmt::Debug;

use windivert_sys::address::WINDIVERT_ADDRESS;

#[derive(Debug)]
pub(crate) struct WinDivertRawPacket {
    pub address: WINDIVERT_ADDRESS,
    pub data: Vec<u8>,
}

#[derive(Debug)]
/// Packet type returned by [`recv`](fn@super::WinDivert::recv) function.
pub enum WinDivertPacket {
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

impl From<WinDivertRawPacket> for WinDivertPacket {
    fn from(packet: WinDivertRawPacket) -> Self {
        match packet.address.layer() {
            windivert_sys::WinDivertLayer::Network | windivert_sys::WinDivertLayer::Forward => {
                WinDivertPacket::Network {
                    addr: WinDivertNetworkData {
                        data: packet.address,
                    },
                    data: packet.data,
                }
            }
            windivert_sys::WinDivertLayer::Flow => WinDivertPacket::Flow {
                addr: WinDivertFlowData {
                    data: packet.address,
                },
            },
            windivert_sys::WinDivertLayer::Socket => WinDivertPacket::Socket {
                addr: WinDivertSocketData {
                    data: packet.address,
                },
            },
            windivert_sys::WinDivertLayer::Reflect => WinDivertPacket::Reflect {
                addr: WinDivertReflectData {
                    data: packet.address,
                },
                filter: packet.data,
            },
        }
    }
}

impl From<WinDivertPacket> for WinDivertRawPacket {
    fn from(packet: WinDivertPacket) -> Self {
        let (buffer, addr) = match packet {
            WinDivertPacket::Network { addr, data } => (data, addr.data),
            WinDivertPacket::Flow { addr } => (Vec::new(), addr.data),
            WinDivertPacket::Socket { addr } => (Vec::new(), addr.data),
            WinDivertPacket::Reflect { addr, filter } => (filter, addr.data),
        };
        WinDivertRawPacket {
            address: addr,
            data: buffer,
        }
    }
}
