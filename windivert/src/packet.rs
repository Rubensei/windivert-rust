use super::address::*;
use std::{borrow::Cow, fmt::Debug};

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
    pub fn parse<'a>(self) -> WinDivertParsedPacket<'a> {
        match self.address.layer() {
            windivert_sys::WinDivertLayer::Network | windivert_sys::WinDivertLayer::Forward => {
                WinDivertParsedPacket::Network {
                    addr: WinDivertNetworkData {
                        data: Cow::Owned(self.address),
                    },
                    data: self.data,
                }
            }
            windivert_sys::WinDivertLayer::Flow => WinDivertParsedPacket::Flow {
                addr: WinDivertFlowData {
                    data: Cow::Owned(self.address),
                },
            },
            windivert_sys::WinDivertLayer::Socket => WinDivertParsedPacket::Socket {
                addr: WinDivertSocketData {
                    data: Cow::Owned(self.address),
                },
            },
            windivert_sys::WinDivertLayer::Reflect => WinDivertParsedPacket::Reflect {
                addr: WinDivertReflectData {
                    data: Cow::Owned(self.address),
                },
                filter: self.data,
            },
        }
    }

    /// Parse a borrowed raw packet
    pub fn parse_slice<'a>(&'a self) -> WinDivertParsedSlice<'a> {
        match self.address.layer() {
            windivert_sys::WinDivertLayer::Network | windivert_sys::WinDivertLayer::Forward => {
                WinDivertParsedSlice::Network {
                    addr: WinDivertNetworkData {
                        data: Cow::Borrowed(&self.address),
                    },
                    data: &self.data,
                }
            }
            windivert_sys::WinDivertLayer::Flow => WinDivertParsedSlice::Flow {
                addr: WinDivertFlowData {
                    data: Cow::Borrowed(&self.address),
                },
            },
            windivert_sys::WinDivertLayer::Socket => WinDivertParsedSlice::Socket {
                addr: WinDivertSocketData {
                    data: Cow::Borrowed(&self.address),
                },
            },
            windivert_sys::WinDivertLayer::Reflect => WinDivertParsedSlice::Reflect {
                addr: WinDivertReflectData {
                    data: Cow::Borrowed(&self.address),
                },
                filter: &self.data,
            },
        }
    }
}

#[derive(Debug)]
/// Parsed packet type.
pub enum WinDivertParsedPacket<'a> {
    /// Packet type returned by handles using [`WinDivertLayer::Network`](super::WinDivertLayer::Network).
    Network {
        /// WinDivert data associated with the packet.
        addr: WinDivertNetworkData<'a>,
        /// Raw captured data.
        data: Vec<u8>,
    },
    /// Packet type returned by handles using [`WinDivertLayer::Flow`](super::WinDivertLayer::Flow).
    Flow {
        /// WinDivert data associated with the packet.
        addr: WinDivertFlowData<'a>,
    },
    /// Packet type returned by handles using [`WinDivertLayer::Socket`](super::WinDivertLayer::Socket).
    Socket {
        /// WinDivert data associated with the packet.
        addr: WinDivertSocketData<'a>,
    },
    /// Packet type returned by handles using [`WinDivertLayer::Reflect`](super::WinDivertLayer::Reflect).
    Reflect {
        /// WinDivert data associated with the packet.
        addr: WinDivertReflectData<'a>,
        /// Object string representation of the filter used to open the handle.
        filter: Vec<u8>,
    },
}

#[derive(Debug)]
/// Parsed slice
pub enum WinDivertParsedSlice<'a> {
    /// Packet type returned by handles using [`WinDivertLayer::Network`](super::WinDivertLayer::Network).
    Network {
        /// WinDivert data associated with the packet.
        addr: WinDivertNetworkData<'a>,
        /// Raw captured data.
        data: &'a Vec<u8>,
    },
    /// Packet type returned by handles using [`WinDivertLayer::Flow`](super::WinDivertLayer::Flow).
    Flow {
        /// WinDivert data associated with the packet.
        addr: WinDivertFlowData<'a>,
    },
    /// Packet type returned by handles using [`WinDivertLayer::Socket`](super::WinDivertLayer::Socket).
    Socket {
        /// WinDivert data associated with the packet.
        addr: WinDivertSocketData<'a>,
    },
    /// Packet type returned by handles using [`WinDivertLayer::Reflect`](super::WinDivertLayer::Reflect).
    Reflect {
        /// WinDivert data associated with the packet.
        addr: WinDivertReflectData<'a>,
        /// Object string representation of the filter used to open the handle.
        filter: &'a Vec<u8>,
    },
}

impl<'a> From<WinDivertParsedPacket<'a>> for WinDivertPacket {
    fn from(packet: WinDivertParsedPacket) -> Self {
        let (buffer, addr) = match packet {
            WinDivertParsedPacket::Network { addr, data } => (data, addr.data),
            WinDivertParsedPacket::Flow { addr } => (Vec::new(), addr.data),
            WinDivertParsedPacket::Socket { addr } => (Vec::new(), addr.data),
            WinDivertParsedPacket::Reflect { addr, filter } => (filter, addr.data),
        };
        WinDivertPacket {
            address: addr.into_owned(),
            data: buffer,
        }
    }
}
