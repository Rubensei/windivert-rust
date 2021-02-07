use super::address::*;
use std::fmt::Debug;

use windivert_sys::address::WINDIVERT_ADDRESS;

#[derive(Debug)]
pub(crate) struct WinDivertRawPacket {
    pub address: WINDIVERT_ADDRESS,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum WinDivertPacket {
    Network {
        addr: WinDivertNetworkData,
        data: Vec<u8>,
    },
    Flow {
        addr: WinDivertFlowData,
    },
    Socket {
        addr: WinDivertSocketData,
    },
    Reflect {
        addr: WinDivertReflectData,
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
