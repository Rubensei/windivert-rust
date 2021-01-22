use super::BitfieldUnit;
use super::WinDivertLayer;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct WINDIVERT_DATA_NETWORK {
    pub IfIdx: u32,
    pub SubIfIdx: u32,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct WINDIVERT_DATA_FLOW {
    pub EndpointId: u64,
    pub ParentEndpointId: u64,
    pub ProcessId: u32,
    pub LocalAddr: [u32; 4usize],
    pub RemoteAddr: [u32; 4usize],
    pub LocalPort: u16,
    pub RemotePort: u16,
    pub Protocol: u8,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct WINDIVERT_DATA_SOCKET {
    pub EndpointId: u64,
    pub ParentEndpointId: u64,
    pub ProcessId: u32,
    pub LocalAddr: [u32; 4usize],
    pub RemoteAddr: [u32; 4usize],
    pub LocalPort: u16,
    pub RemotePort: u16,
    pub Protocol: u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct WINDIVERT_DATA_REFLECT {
    pub Timestamp: i64,
    pub ProcessId: u32,
    pub Layer: WinDivertLayer,
    pub Flags: u64,
    pub Priority: i16,
}

impl Default for WINDIVERT_DATA_REFLECT {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union WINDIVERT_ADDRESS_UNION_FIELD {
    pub Network: WINDIVERT_DATA_NETWORK,
    pub Flow: WINDIVERT_DATA_FLOW,
    pub Socket: WINDIVERT_DATA_SOCKET,
    pub Reflect: WINDIVERT_DATA_REFLECT,
    pub reserved: [u8; 64usize],
    _union_align: [u64; 8usize],
}

impl Default for WINDIVERT_ADDRESS_UNION_FIELD {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct WINDIVERT_ADDRESS {
    pub Timestamp: i64,
    pub addr_bitfield: BitfieldUnit<[u8; 4usize], u8>,
    pub reserved: u32,
    pub union_field: WINDIVERT_ADDRESS_UNION_FIELD,
}

impl Default for WINDIVERT_ADDRESS {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

impl WINDIVERT_ADDRESS {
    #[inline]
    pub fn Layer(&self) -> u32 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(0usize, 8u8) as u32) }
    }
    #[inline]
    pub fn set_Layer(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::std::mem::transmute(val);
            self.addr_bitfield.set(0usize, 8u8, val as u64)
        }
    }
    #[inline]
    pub fn Event(&self) -> u32 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(8usize, 8u8) as u32) }
    }
    #[inline]
    pub fn set_Event(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::std::mem::transmute(val);
            self.addr_bitfield.set(8usize, 8u8, val as u64)
        }
    }
    #[inline]
    pub fn Sniffed(&self) -> u32 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(16usize, 1u8) as u32) }
    }
    #[inline]
    pub fn set_Sniffed(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::std::mem::transmute(val);
            self.addr_bitfield.set(16usize, 1u8, val as u64)
        }
    }
    #[inline]
    pub fn Outbound(&self) -> u32 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(17usize, 1u8) as u32) }
    }
    #[inline]
    pub fn set_Outbound(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::std::mem::transmute(val);
            self.addr_bitfield.set(17usize, 1u8, val as u64)
        }
    }
    #[inline]
    pub fn Loopback(&self) -> u32 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(18usize, 1u8) as u32) }
    }
    #[inline]
    pub fn set_Loopback(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::std::mem::transmute(val);
            self.addr_bitfield.set(18usize, 1u8, val as u64)
        }
    }
    #[inline]
    pub fn Impostor(&self) -> u32 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(19usize, 1u8) as u32) }
    }
    #[inline]
    pub fn set_Impostor(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::std::mem::transmute(val);
            self.addr_bitfield.set(19usize, 1u8, val as u64)
        }
    }
    #[inline]
    pub fn IPv6(&self) -> u32 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(20usize, 1u8) as u32) }
    }
    #[inline]
    pub fn set_IPv6(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::std::mem::transmute(val);
            self.addr_bitfield.set(20usize, 1u8, val as u64)
        }
    }
    #[inline]
    pub fn IPChecksum(&self) -> u32 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(21usize, 1u8) as u32) }
    }
    #[inline]
    pub fn set_IPChecksum(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::std::mem::transmute(val);
            self.addr_bitfield.set(21usize, 1u8, val as u64)
        }
    }
    #[inline]
    pub fn TCPChecksum(&self) -> u32 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(22usize, 1u8) as u32) }
    }
    #[inline]
    pub fn set_TCPChecksum(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::std::mem::transmute(val);
            self.addr_bitfield.set(22usize, 1u8, val as u64)
        }
    }
    #[inline]
    pub fn UDPChecksum(&self) -> u32 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(23usize, 1u8) as u32) }
    }
    #[inline]
    pub fn set_UDPChecksum(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::std::mem::transmute(val);
            self.addr_bitfield.set(23usize, 1u8, val as u64)
        }
    }
    #[inline]
    pub fn Reserved1(&self) -> u32 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(24usize, 8u8) as u32) }
    }
    #[inline]
    pub fn set_Reserved1(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::std::mem::transmute(val);
            self.addr_bitfield.set(24usize, 8u8, val as u64)
        }
    }
    #[inline]
    pub fn newaddr_bitfield(
        Layer: u32,
        Event: u32,
        Sniffed: u32,
        Outbound: u32,
        Loopback: u32,
        Impostor: u32,
        IPv6: u32,
        IPChecksum: u32,
        TCPChecksum: u32,
        UDPChecksum: u32,
        Reserved1: u32,
    ) -> BitfieldUnit<[u8; 4usize], u8> {
        let mut bitfield_unit: BitfieldUnit<[u8; 4usize], u8> = Default::default();
        bitfield_unit.set(0usize, 8u8, {
            let Layer: u32 = unsafe { ::std::mem::transmute(Layer) };
            Layer as u64
        });
        bitfield_unit.set(8usize, 8u8, {
            let Event: u32 = unsafe { ::std::mem::transmute(Event) };
            Event as u64
        });
        bitfield_unit.set(16usize, 1u8, {
            let Sniffed: u32 = unsafe { ::std::mem::transmute(Sniffed) };
            Sniffed as u64
        });
        bitfield_unit.set(17usize, 1u8, {
            let Outbound: u32 = unsafe { ::std::mem::transmute(Outbound) };
            Outbound as u64
        });
        bitfield_unit.set(18usize, 1u8, {
            let Loopback: u32 = unsafe { ::std::mem::transmute(Loopback) };
            Loopback as u64
        });
        bitfield_unit.set(19usize, 1u8, {
            let Impostor: u32 = unsafe { ::std::mem::transmute(Impostor) };
            Impostor as u64
        });
        bitfield_unit.set(20usize, 1u8, {
            let IPv6: u32 = unsafe { ::std::mem::transmute(IPv6) };
            IPv6 as u64
        });
        bitfield_unit.set(21usize, 1u8, {
            let IPChecksum: u32 = unsafe { ::std::mem::transmute(IPChecksum) };
            IPChecksum as u64
        });
        bitfield_unit.set(22usize, 1u8, {
            let TCPChecksum: u32 = unsafe { ::std::mem::transmute(TCPChecksum) };
            TCPChecksum as u64
        });
        bitfield_unit.set(23usize, 1u8, {
            let UDPChecksum: u32 = unsafe { ::std::mem::transmute(UDPChecksum) };
            UDPChecksum as u64
        });
        bitfield_unit.set(24usize, 8u8, {
            let Reserved1: u32 = unsafe { ::std::mem::transmute(Reserved1) };
            Reserved1 as u64
        });
        bitfield_unit
    }
}
