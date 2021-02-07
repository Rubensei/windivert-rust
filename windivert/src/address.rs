use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::{WinDivertEvent, WinDivertFlags, WinDivertLayer};
use windivert_sys::address::*;

macro_rules! addr_impl {
    () => {
        pub fn event_timestamp(&self) -> i64 {
            self.data.timestamp
        }

        pub fn event(&self) -> WinDivertEvent {
            self.data.event()
        }

        pub fn event_layer(&self) -> WinDivertLayer {
            self.data.layer()
        }

        pub fn sniffed(&self) -> bool {
            self.data.sniffed()
        }

        pub fn outbound(&self) -> bool {
            self.data.outbound()
        }

        pub fn set_outbound(&mut self, value: bool) {
            self.data.set_outbound(value)
        }

        pub fn loopback(&self) -> bool {
            self.data.loopback()
        }

        pub fn impostor(&self) -> bool {
            self.data.impostor()
        }

        pub fn set_impostor(&mut self, value: bool) {
            self.data.set_impostor(value)
        }

        pub fn ipv6(&self) -> bool {
            self.data.ipv6()
        }

        pub fn ip_checksum(&self) -> bool {
            self.data.ipchecksum()
        }

        pub fn set_ip_checksum(&mut self, value: bool) {
            self.data.set_ipchecksum(value)
        }

        pub fn tcp_checksum(&self) -> bool {
            self.data.tcpchecksum()
        }

        pub fn set_tcp_checksum(&mut self, value: bool) {
            self.data.set_tcpchecksum(value)
        }

        pub fn udp_checksum(&self) -> bool {
            self.data.udpchecksum()
        }

        pub fn set_udp_checksum(&mut self, value: bool) {
            self.data.set_udpchecksum(value)
        }
    };
}

#[derive(Debug, Default)]
pub struct WinDivertNetworkData {
    pub(crate) data: WINDIVERT_ADDRESS,
}

impl WinDivertNetworkData {
    addr_impl!();

    fn data(&self) -> &WINDIVERT_DATA_NETWORK {
        unsafe { &self.data.union_field.Network }
    }

    fn data_mut(&mut self) -> &mut WINDIVERT_DATA_NETWORK {
        unsafe { &mut self.data.union_field.Network }
    }

    pub fn interface_id(&self) -> u32 {
        self.data().interface_id
    }

    pub fn set_interface_id(&mut self, value: u32) {
        self.data_mut().interface_id = value
    }

    pub fn subinterface_id(&self) -> u32 {
        self.data().subinterface_id
    }

    pub fn set_subinterface_id(&mut self, value: u32) {
        self.data_mut().subinterface_id = value
    }
}

#[derive(Debug, Default)]
pub struct WinDivertFlowData {
    pub(crate) data: WINDIVERT_ADDRESS,
}

pub type WinDivertSocketData = WinDivertFlowData;

impl WinDivertFlowData {
    addr_impl!();

    fn data(&self) -> &WINDIVERT_DATA_FLOW {
        unsafe { &self.data.union_field.Flow }
    }

    pub fn endpoint_id(&self) -> u64 {
        self.data().endpoint_id
    }

    pub fn parent_endpoint_id(&self) -> u64 {
        self.data().parent_endpoint_id
    }

    pub fn process_id(&self) -> u32 {
        self.data().process_id
    }

    pub fn local_address(&self) -> IpAddr {
        if self.data.ipv6() {
            IpAddr::V6(Ipv6Addr::from(
                self.data()
                    .local_addr
                    .iter()
                    .rev()
                    .fold(0u128, |acc, &x| acc << 32 | (x as u128)),
            ))
        } else {
            IpAddr::V4(Ipv4Addr::from(self.data().local_addr[0]))
        }
    }

    pub fn remote_address(&self) -> IpAddr {
        if self.data.ipv6() {
            IpAddr::V6(Ipv6Addr::from(
                self.data()
                    .remote_addr
                    .iter()
                    .rev()
                    .fold(0u128, |acc, &x| acc << 32 | (x as u128)),
            ))
        } else {
            IpAddr::V4(Ipv4Addr::from(self.data().remote_addr[0]))
        }
    }

    pub fn local_port(&self) -> u16 {
        self.data().local_port
    }

    pub fn remote_port(&self) -> u16 {
        self.data().remote_port
    }

    pub fn protocol(&self) -> u8 {
        self.data().protocol
    }
}

#[derive(Debug)]
pub struct WinDivertReflectData {
    pub(crate) data: WINDIVERT_ADDRESS,
}

impl WinDivertReflectData {
    addr_impl!();

    fn data(&self) -> &WINDIVERT_DATA_REFLECT {
        unsafe { &self.data.union_field.Reflect }
    }

    pub fn timestamp(&self) -> i64 {
        self.data().timestamp
    }

    pub fn process_id(&self) -> u32 {
        self.data().process_id
    }

    pub fn layer(&self) -> WinDivertLayer {
        self.data().layer
    }

    pub fn flags(&self) -> WinDivertFlags {
        self.data().flags
    }

    pub fn priority(&self) -> i16 {
        self.data().priority
    }
}
