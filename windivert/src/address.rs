use std::{
    marker::PhantomData,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

use crate::{layer, prelude::*};
use windivert_sys::address::*;

/// Newtype wrapper around [`WINDIVERT_ADDRESS`] using typestate to provide a safe interface.
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct WinDivertAddress<L: layer::WinDivertLayerTrait> {
    data: WINDIVERT_ADDRESS,
    _layer: PhantomData<L>,
}

impl<L: layer::WinDivertLayerTrait> WinDivertAddress<L> {
    pub(crate) fn from_raw(data: WINDIVERT_ADDRESS) -> Self {
        Self {
            data,
            _layer: PhantomData,
        }
    }

    /// Timestamp of the event. Uses same clock as `QueryPerformanceCounter()`
    pub fn event_timestamp(&self) -> i64 {
        self.data.timestamp
    }

    /// Type of captured event
    pub fn event(&self) -> WinDivertEvent {
        self.data.event()
    }

    /// The handle's layer
    pub fn event_layer(&self) -> WinDivertLayer {
        self.data.layer()
    }

    /// Set to `true` if the event was sniffed (i.e., not blocked), `false` otherwise
    pub fn sniffed(&self) -> bool {
        self.data.sniffed()
    }

    /// Set to `true` for outbound packets/event, `false` for inbound or otherwise
    pub fn outbound(&self) -> bool {
        self.data.outbound()
    }

    /// Outbound setter
    pub fn set_outbound(&mut self, value: bool) {
        self.data.set_outbound(value)
    }

    /// Set to `true` for loopback packets, `false` otherwise
    pub fn loopback(&self) -> bool {
        self.data.loopback()
    }

    ///  Set to `true` for impostor packets, `false` otherwise.
    pub fn impostor(&self) -> bool {
        self.data.impostor()
    }

    /// Impostor setter
    pub fn set_impostor(&mut self, value: bool) {
        self.data.set_impostor(value)
    }

    /// Set to `true` for IPv6 packets/events, `false` otherwise
    pub fn ipv6(&self) -> bool {
        self.data.ipv6()
    }

    /// Set to `true` if the IPv4 checksum is valid, `false` otherwise.
    pub fn ip_checksum(&self) -> bool {
        self.data.ipchecksum()
    }

    /// IPv4 checksum setter
    pub fn set_ip_checksum(&mut self, value: bool) {
        self.data.set_ipchecksum(value)
    }

    /// Set to `true` if the TCP checksum is valid, `false` otherwise.
    pub fn tcp_checksum(&self) -> bool {
        self.data.tcpchecksum()
    }

    /// TCP checksum setter
    pub fn set_tcp_checksum(&mut self, value: bool) {
        self.data.set_tcpchecksum(value)
    }

    /// Set to `true` if the UDP checksum is valid, `false` otherwise.
    pub fn udp_checksum(&self) -> bool {
        self.data.udpchecksum()
    }

    /// UDP checksum setter
    pub fn set_udp_checksum(&mut self, value: bool) {
        self.data.set_udpchecksum(value)
    }
}

impl<L: layer::WinDivertLayerTrait> AsRef<WINDIVERT_ADDRESS> for WinDivertAddress<L> {
    fn as_ref(&self) -> &WINDIVERT_ADDRESS {
        &self.data
    }
}

impl WinDivertAddress<layer::NetworkLayer> {
    /// Create a new [`WinDivertData`] to inject new packets.
    /// SAFETY: The default value for address is zeroed memory, caller must fill with valid data before sending.
    pub unsafe fn new() -> Self {
        Self {
            data: Default::default(),
            _layer: PhantomData,
        }
    }

    fn data(&self) -> &WINDIVERT_DATA_NETWORK {
        unsafe { &self.data.union_field.Network }
    }

    fn data_mut(&mut self) -> &mut WINDIVERT_DATA_NETWORK {
        unsafe { &mut self.data.union_field.Network }
    }

    /// The interface index on which the packet arrived (for inbound packets), or is to be sent (for outbound packets)
    pub fn interface_index(&self) -> u32 {
        self.data().interface_id
    }

    /// Interface index setter
    pub fn set_interface_index(&mut self, value: u32) {
        self.data_mut().interface_id = value
    }

    /// The sub-interface index for `interface_id()`
    pub fn subinterface_index(&self) -> u32 {
        self.data().subinterface_id
    }

    /// Sub interface index setter
    pub fn set_subinterface_index(&mut self, value: u32) {
        self.data_mut().subinterface_id = value
    }
}

impl WinDivertAddress<layer::ForwardLayer> {
    /// Create a new [`WinDivertData`] to inject new packets.
    /// SAFETY: The default value for address is zeroed memory, caller must fill with valid data before sending.
    pub unsafe fn new() -> Self {
        Self {
            data: Default::default(),
            _layer: PhantomData,
        }
    }

    fn data(&self) -> &WINDIVERT_DATA_NETWORK {
        unsafe { &self.data.union_field.Network }
    }

    fn data_mut(&mut self) -> &mut WINDIVERT_DATA_NETWORK {
        unsafe { &mut self.data.union_field.Network }
    }

    /// The interface index on which the packet arrived (for inbound packets), or is to be sent (for outbound packets)
    pub fn interface_index(&self) -> u32 {
        self.data().interface_id
    }

    /// Interface index setter
    pub fn set_interface_index(&mut self, value: u32) {
        self.data_mut().interface_id = value
    }

    /// The sub-interface index for `interface_id()`
    pub fn subinterface_index(&self) -> u32 {
        self.data().subinterface_id
    }

    /// Sub interface index setter
    pub fn set_subinterface_index(&mut self, value: u32) {
        self.data_mut().subinterface_id = value
    }
}

impl WinDivertAddress<layer::FlowLayer> {
    fn data(&self) -> &WINDIVERT_DATA_FLOW {
        unsafe { &self.data.union_field.Flow }
    }

    /// The endpoint ID of the flow
    pub fn endpoint_id(&self) -> u64 {
        self.data().endpoint_id
    }

    /// The parent endpoint ID of the flow
    pub fn parent_endpoint_id(&self) -> u64 {
        self.data().parent_endpoint_id
    }

    /// The parent endpoint ID of the flow
    pub fn process_id(&self) -> u32 {
        self.data().process_id
    }

    /// The local address associated with the flow
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

    /// The remote address associated with the flow
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

    /// The locla port associated with the flow
    pub fn local_port(&self) -> u16 {
        self.data().local_port
    }

    /// The remote port associated with the flow
    pub fn remote_port(&self) -> u16 {
        self.data().remote_port
    }

    /// The protocol associated with the flow
    pub fn protocol(&self) -> u8 {
        self.data().protocol
    }
}

impl WinDivertAddress<layer::SocketLayer> {
    fn data(&self) -> &WINDIVERT_DATA_FLOW {
        unsafe { &self.data.union_field.Flow }
    }

    /// The endpoint ID of the flow
    pub fn endpoint_id(&self) -> u64 {
        self.data().endpoint_id
    }

    /// The parent endpoint ID of the flow
    pub fn parent_endpoint_id(&self) -> u64 {
        self.data().parent_endpoint_id
    }

    /// The parent endpoint ID of the flow
    pub fn process_id(&self) -> u32 {
        self.data().process_id
    }

    /// The local address associated with the flow
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

    /// The remote address associated with the flow
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

    /// The locla port associated with the flow
    pub fn local_port(&self) -> u16 {
        self.data().local_port
    }

    /// The remote port associated with the flow
    pub fn remote_port(&self) -> u16 {
        self.data().remote_port
    }

    /// The protocol associated with the flow
    pub fn protocol(&self) -> u8 {
        self.data().protocol
    }
}

impl WinDivertAddress<layer::ReflectLayer> {
    fn data(&self) -> &WINDIVERT_DATA_REFLECT {
        unsafe { &self.data.union_field.Reflect }
    }

    /// A timestamp indicating when the handle was opened
    pub fn timestamp(&self) -> i64 {
        self.data().timestamp
    }

    /// The ID of the process that opened the handle
    pub fn process_id(&self) -> u32 {
        self.data().process_id
    }

    /// The layer of the opened handle
    pub fn layer(&self) -> WinDivertLayer {
        self.data().layer
    }

    /// The flags of the opened handle
    pub fn flags(&self) -> WinDivertFlags {
        self.data().flags
    }

    /// The priority of the opened handle
    pub fn priority(&self) -> i16 {
        self.data().priority
    }
}
