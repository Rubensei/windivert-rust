/*!
WinDivert address types.

For more info, refer to the [docs](https://reqrypt.org/windivert-doc.html#divert_address).
*/

use std::convert::TryFrom;

use super::{BitfieldUnit, WinDivertEvent, WinDivertFlags, WinDivertLayer};

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
/**
Represents the associated data recieved using [`WinDivertLayer::Network`]
*/
pub struct WINDIVERT_DATA_NETWORK {
    /// Interface index on whick the packet arrived (for inbound packets) or will be sent (for outbound packets).
    pub IfIdx: u32,
    /// The sub-interface index for `IfIdx`
    pub SubIfIdx: u32,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
/**
Represents the associated data recieved using [`WinDivertLayer::Flow`]
*/
pub struct WINDIVERT_DATA_FLOW {
    /// The endpoint ID of the flow.
    pub EndpointId: u64,
    /// The parent endpoint ID of the flow.
    pub ParentEndpointId: u64,
    /// The id of the process associated with the flow.
    pub ProcessId: u32,
    /// The local address associated with the flow.
    pub LocalAddr: [u32; 4usize],
    /// The remote address associated with the flow.
    pub RemoteAddr: [u32; 4usize],
    /// The local port associated with the flow.
    pub LocalPort: u16,
    /// The remote port associated with the flow.
    pub RemotePort: u16,
    /// The flow protocol.
    pub Protocol: u8,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
/**
Represents the associated data recieved using [`WinDivertLayer::Socket`]
*/
pub struct WINDIVERT_DATA_SOCKET {
    /// The endpoint ID of the socket.
    pub EndpointId: u64,
    /// The parent endpoint ID of the socket.
    pub ParentEndpointId: u64,
    /// The id of the process associated with the socket.
    pub ProcessId: u32,
    /// The local address associated with the socket.
    pub LocalAddr: [u32; 4usize],
    /// The remote address associated with the socket.
    pub RemoteAddr: [u32; 4usize],
    /// The local port associated with the socket.
    pub LocalPort: u16,
    /// The remote port associated with the socket.
    pub RemotePort: u16,
    /// The socket protocol.
    pub Protocol: u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
/**
Represents the associated data recieved using [`WinDivertLayer::Reflect`]
*/
pub struct WINDIVERT_DATA_REFLECT {
    ///
    pub Timestamp: i64,
    pub ProcessId: u32,
    /// [`WinDivertLayer`] parameter on [`WinDivertOpen`](super::WinDivertOpen) for the specified handle.
    pub Layer: WinDivertLayer,
    /// [`WinDivertFlags`] parameter on [`WinDivertOpen`](super::WinDivertOpen) for the specified handle.
    pub Flags: WinDivertFlags,
    /// Priority parameter on [`WinDivertOpen`](super::WinDivertOpen) for the specified handle.
    pub Priority: i16,
}

impl Default for WINDIVERT_DATA_REFLECT {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
/// Union of the different data types associated with the possible layer values.
pub union WINDIVERT_ADDRESS_UNION_FIELD {
    pub Network: WINDIVERT_DATA_NETWORK,
    pub Flow: WINDIVERT_DATA_FLOW,
    pub Socket: WINDIVERT_DATA_SOCKET,
    pub Reflect: WINDIVERT_DATA_REFLECT,
    reserved: [u8; 64usize],
    _union_align: [u64; 8usize],
}

impl Default for WINDIVERT_ADDRESS_UNION_FIELD {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
/// Base data type returned by [`recv`](fn@super::WinDivertRecv) and required by [`send`](fn@super::WinDivertSend)
pub struct WINDIVERT_ADDRESS {
    /// Timestamp indicating when the event occurred.
    pub Timestamp: i64,
    addr_bitfield: BitfieldUnit<[u8; 4usize], u8>,
    reserved: u32,
    /// Union of the different data types associated with the possible layer values.
    pub union_field: WINDIVERT_ADDRESS_UNION_FIELD,
}

impl Default for WINDIVERT_ADDRESS {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

impl WINDIVERT_ADDRESS {
    #[inline]
    /// Getter for the handle [`layer`](super::WinDivertLayer)
    pub fn Layer(&self) -> WinDivertLayer {
        WinDivertLayer::try_from(self.addr_bitfield.get(0usize, 8u8) as u32)
            .expect("Layer always is correct since it would have produced an error in Open()")
    }
    #[inline]
    /// Setter for the handle [`layer`](super::WinDivertLayer)
    pub fn set_Layer(&mut self, val: WinDivertLayer) {
        self.addr_bitfield.set(0usize, 8u8, u32::from(val) as u64)
    }
    #[inline]
    /// Getter for the handle [`event`](super::WinDivertEvent)
    pub fn Event(&self) -> WinDivertEvent {
        WinDivertEvent::try_from(self.addr_bitfield.get(8usize, 8u8) as u8)
            .expect("Event always is correct since teh value comes from the DLL functions.")
    }
    #[inline]
    /// Setter for the handle [`event`](super::WinDivertEvent)
    pub fn set_Event(&mut self, val: u32) {
        self.addr_bitfield.set(8usize, 8u8, u32::from(val) as u64)
    }
    #[inline]
    /// Set to true if the packet was sniffed (not blocked).
    pub fn Sniffed(&self) -> bool {
        self.addr_bitfield.get(16usize, 1u8) == 1
    }
    #[inline]
    /// Sniffed flag setter.
    pub fn set_Sniffed(&mut self, val: bool) {
        self.addr_bitfield.set(16usize, 1u8, val as u64)
    }
    #[inline]
    /// Set to true for outbound packet events.
    pub fn Outbound(&self) -> bool {
        self.addr_bitfield.get(17usize, 1u8) == 1
    }
    #[inline]
    /// Outbound flag setter.
    pub fn set_Outbound(&mut self, val: bool) {
        self.addr_bitfield.set(17usize, 1u8, val as u64)
    }
    #[inline]
    /// Set to true for loopback packets.
    pub fn Loopback(&self) -> bool {
        self.addr_bitfield.get(18usize, 1u8) == 1
    }
    #[inline]
    /// Loopback flag setter.
    pub fn set_Loopback(&mut self, val: bool) {
        self.addr_bitfield.set(18usize, 1u8, val as u64)
    }
    #[inline]
    /// Set to true for "impostor" packets.
    pub fn Impostor(&self) -> bool {
        self.addr_bitfield.get(19usize, 1u8) == 1
    }
    #[inline]
    /// Impostor flag setter.
    pub fn set_Impostor(&mut self, val: bool) {
        self.addr_bitfield.set(19usize, 1u8, val as u64)
    }
    #[inline]
    /// Set to true for IPv6 packets.
    pub fn IPv6(&self) -> bool {
        self.addr_bitfield.get(20usize, 1u8) == 1
    }
    #[inline]
    /// IPv6 flag setter.
    pub fn set_IPv6(&mut self, val: bool) {
        self.addr_bitfield.set(20usize, 1u8, val as u64)
    }
    #[inline]
    /// Set to true if the IPv4 checksum is valid.
    pub fn IPChecksum(&self) -> bool {
        self.addr_bitfield.get(21usize, 1u8) == 1
    }
    #[inline]
    /// IPv4 checksum flag setter.
    pub fn set_IPChecksum(&mut self, val: bool) {
        self.addr_bitfield.set(21usize, 1u8, val as u64)
    }
    #[inline]
    /// Set to true if the TCP checksum is valid.
    pub fn TCPChecksum(&self) -> bool {
        self.addr_bitfield.get(22usize, 1u8) == 1
    }
    #[inline]
    /// TCP checksum flag setter.
    pub fn set_TCPChecksum(&mut self, val: bool) {
        self.addr_bitfield.set(22usize, 1u8, val as u64)
    }
    #[inline]
    /// Set to true if the UDP checksum is valid.
    pub fn UDPChecksum(&self) -> bool {
        self.addr_bitfield.get(23usize, 1u8) == 1
    }
    #[inline]
    /// UDP checksum flag setter.
    pub fn set_UDPChecksum(&mut self, val: bool) {
        self.addr_bitfield.set(23usize, 1u8, val as u64)
    }
}
