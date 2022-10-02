use super::address::*;
use std::{borrow::Cow, fmt::Debug};

use windivert_sys::address::WINDIVERT_ADDRESS;

// TODO: Allow creating packets for injection.
/// Raw packet using an already allocated buffer
#[derive(Debug)]
pub struct WinDivertPacket<'a> {
    pub(crate) address: WINDIVERT_ADDRESS,
    /// Raw captured data
    pub data: Cow<'a, [u8]>,
}

impl<'a> WinDivertPacket<'a> {
    /// Parse a raw packet
    pub fn parse(self) -> WinDivertParsedPacket<'a> {
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
                data: self.data,
            },
        }
    }

    ///
    pub fn into_owned(self) -> WinDivertPacket<'static> {
        WinDivertPacket {
            address: self.address.clone(),
            data: self.data.into_owned().into(),
        }
    }
}

#[derive(Debug)]
/// Parsed packet type.
pub enum WinDivertParsedPacket<'a> {
    /// Packet type returned by handles using [`WinDivertLayer::Network`](super::WinDivertLayer::Network).
    Network {
        /// WinDivert data associated with the packet.
        addr: WinDivertNetworkData,
        /// Raw captured data.
        data: Cow<'a, [u8]>,
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
        data: Cow<'a, [u8]>,
    },
}

impl<'a> From<WinDivertParsedPacket<'a>> for WinDivertPacket<'a> {
    fn from(packet: WinDivertParsedPacket<'a>) -> Self {
        let (buffer, addr) = match packet {
            WinDivertParsedPacket::Network { addr, data } => (data, addr.data),
            WinDivertParsedPacket::Flow { addr } => (Cow::default(), addr.data),
            WinDivertParsedPacket::Socket { addr } => (Cow::default(), addr.data),
            WinDivertParsedPacket::Reflect { addr, data } => (data, addr.data),
        };
        WinDivertPacket {
            address: addr,
            data: buffer,
        }
    }
}
