/*!
WinDivert header types.
*/

use super::BitfieldUnit;

/**
IPV4 header.

For more info, refer to the [docs](https://reqrypt.org/windivert-doc.html#divert_iphdr)
*/
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct WINDIVERT_IPHDR {
    pub addr_bitfield: BitfieldUnit<[u8; 1usize], u8>,
    pub TOS: u8,
    pub Length: u16,
    pub Id: u16,
    pub FragOff0: u16,
    pub TTL: u8,
    pub Protocol: u8,
    pub Checksum: u16,
    pub SrcAddr: u32,
    pub DstAddr: u32,
}

impl WINDIVERT_IPHDR {
    #[inline]
    pub fn HdrLength(&self) -> u8 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(0usize, 4u8) as u8) }
    }
    #[inline]
    pub fn set_HdrLength(&mut self, val: u8) {
        unsafe {
            let val: u8 = ::std::mem::transmute(val);
            self.addr_bitfield.set(0usize, 4u8, val as u64)
        }
    }
    #[inline]
    pub fn Version(&self) -> u8 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(4usize, 4u8) as u8) }
    }
    #[inline]
    pub fn set_Version(&mut self, val: u8) {
        unsafe {
            let val: u8 = ::std::mem::transmute(val);
            self.addr_bitfield.set(4usize, 4u8, val as u64)
        }
    }
    #[inline]
    pub fn newaddr_bitfield(HdrLength: u8, Version: u8) -> BitfieldUnit<[u8; 1usize], u8> {
        let mut bitfield_unit: BitfieldUnit<[u8; 1usize], u8> = Default::default();
        bitfield_unit.set(0usize, 4u8, {
            let HdrLength: u8 = unsafe { ::std::mem::transmute(HdrLength) };
            HdrLength as u64
        });
        bitfield_unit.set(4usize, 4u8, {
            let Version: u8 = unsafe { ::std::mem::transmute(Version) };
            Version as u64
        });
        bitfield_unit
    }
}
/// [IPV4 header](WINDIVERT_IPHDR) pointer type.
pub type PWINDIVERT_IPHDR = *mut WINDIVERT_IPHDR;

/**
IPV6 header.

For more info, refer to the [docs](https://reqrypt.org/windivert-doc.html#divert_ipv6hdr)
*/
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct WINDIVERT_IPV6HDR {
    pub addr_bitfield: BitfieldUnit<[u8; 2usize], u8>,
    pub FlowLabel1: u16,
    pub Length: u16,
    pub NextHdr: u8,
    pub HopLimit: u8,
    pub SrcAddr: [u32; 4usize],
    pub DstAddr: [u32; 4usize],
}

impl WINDIVERT_IPV6HDR {
    #[inline]
    pub fn TrafficClass0(&self) -> u8 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(0usize, 4u8) as u8) }
    }
    #[inline]
    pub fn set_TrafficClass0(&mut self, val: u8) {
        unsafe {
            let val: u8 = ::std::mem::transmute(val);
            self.addr_bitfield.set(0usize, 4u8, val as u64)
        }
    }
    #[inline]
    pub fn Version(&self) -> u8 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(4usize, 4u8) as u8) }
    }
    #[inline]
    pub fn set_Version(&mut self, val: u8) {
        unsafe {
            let val: u8 = ::std::mem::transmute(val);
            self.addr_bitfield.set(4usize, 4u8, val as u64)
        }
    }
    #[inline]
    pub fn FlowLabel0(&self) -> u8 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(8usize, 4u8) as u8) }
    }
    #[inline]
    pub fn set_FlowLabel0(&mut self, val: u8) {
        unsafe {
            let val: u8 = ::std::mem::transmute(val);
            self.addr_bitfield.set(8usize, 4u8, val as u64)
        }
    }
    #[inline]
    pub fn TrafficClass1(&self) -> u8 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(12usize, 4u8) as u8) }
    }
    #[inline]
    pub fn set_TrafficClass1(&mut self, val: u8) {
        unsafe {
            let val: u8 = ::std::mem::transmute(val);
            self.addr_bitfield.set(12usize, 4u8, val as u64)
        }
    }
    #[inline]
    pub fn newaddr_bitfield(
        TrafficClass0: u8,
        Version: u8,
        FlowLabel0: u8,
        TrafficClass1: u8,
    ) -> BitfieldUnit<[u8; 2usize], u8> {
        let mut bitfield_unit: BitfieldUnit<[u8; 2usize], u8> = Default::default();
        bitfield_unit.set(0usize, 4u8, {
            let TrafficClass0: u8 = unsafe { ::std::mem::transmute(TrafficClass0) };
            TrafficClass0 as u64
        });
        bitfield_unit.set(4usize, 4u8, {
            let Version: u8 = unsafe { ::std::mem::transmute(Version) };
            Version as u64
        });
        bitfield_unit.set(8usize, 4u8, {
            let FlowLabel0: u8 = unsafe { ::std::mem::transmute(FlowLabel0) };
            FlowLabel0 as u64
        });
        bitfield_unit.set(12usize, 4u8, {
            let TrafficClass1: u8 = unsafe { ::std::mem::transmute(TrafficClass1) };
            TrafficClass1 as u64
        });
        bitfield_unit
    }
}
/// [IPV6 header](WINDIVERT_IPV6HDR) pointer type.
pub type PWINDIVERT_IPV6HDR = *mut WINDIVERT_IPV6HDR;

/**
ICMP header.

For more info, refer to the [docs](https://reqrypt.org/windivert-doc.html#divert_icmphdr)
*/
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct WINDIVERT_ICMPHDR {
    pub Type: u8,
    pub Code: u8,
    pub Checksum: u16,
    pub Body: u32,
}

/// [ICMP header](WINDIVERT_ICMPHDR) pointer type.
pub type PWINDIVERT_ICMPHDR = *mut WINDIVERT_ICMPHDR;

/**
ICMPV6 header.

For more info, refer to the [docs](https://reqrypt.org/windivert-doc.html#divert_icmpv6hdr)
*/
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct WINDIVERT_ICMPV6HDR {
    pub Type: u8,
    pub Code: u8,
    pub Checksum: u16,
    pub Body: u32,
}

/// [ICMPV6 header](WINDIVERT_ICMPV6HDR) pointer type.
pub type PWINDIVERT_ICMPV6HDR = *mut WINDIVERT_ICMPV6HDR;

/**
TCP header.

For more info, refer to the [docs](https://reqrypt.org/windivert-doc.html#divert_tcphdr)
*/
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct WINDIVERT_TCPHDR {
    pub SrcPort: u16,
    pub DstPort: u16,
    pub SeqNum: u32,
    pub AckNum: u32,
    pub addr_bitfield: BitfieldUnit<[u8; 2usize], u8>,
    pub Window: u16,
    pub Checksum: u16,
    pub UrgPtr: u16,
}

impl WINDIVERT_TCPHDR {
    #[inline]
    pub fn Reserved1(&self) -> u16 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(0usize, 4u8) as u16) }
    }
    #[inline]
    pub fn set_Reserved1(&mut self, val: u16) {
        unsafe {
            let val: u16 = ::std::mem::transmute(val);
            self.addr_bitfield.set(0usize, 4u8, val as u64)
        }
    }
    #[inline]
    pub fn HdrLength(&self) -> u16 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(4usize, 4u8) as u16) }
    }
    #[inline]
    pub fn set_HdrLength(&mut self, val: u16) {
        unsafe {
            let val: u16 = ::std::mem::transmute(val);
            self.addr_bitfield.set(4usize, 4u8, val as u64)
        }
    }
    #[inline]
    pub fn Fin(&self) -> u16 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(8usize, 1u8) as u16) }
    }
    #[inline]
    pub fn set_Fin(&mut self, val: u16) {
        unsafe {
            let val: u16 = ::std::mem::transmute(val);
            self.addr_bitfield.set(8usize, 1u8, val as u64)
        }
    }
    #[inline]
    pub fn Syn(&self) -> u16 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(9usize, 1u8) as u16) }
    }
    #[inline]
    pub fn set_Syn(&mut self, val: u16) {
        unsafe {
            let val: u16 = ::std::mem::transmute(val);
            self.addr_bitfield.set(9usize, 1u8, val as u64)
        }
    }
    #[inline]
    pub fn Rst(&self) -> u16 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(10usize, 1u8) as u16) }
    }
    #[inline]
    pub fn set_Rst(&mut self, val: u16) {
        unsafe {
            let val: u16 = ::std::mem::transmute(val);
            self.addr_bitfield.set(10usize, 1u8, val as u64)
        }
    }
    #[inline]
    pub fn Psh(&self) -> u16 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(11usize, 1u8) as u16) }
    }
    #[inline]
    pub fn set_Psh(&mut self, val: u16) {
        unsafe {
            let val: u16 = ::std::mem::transmute(val);
            self.addr_bitfield.set(11usize, 1u8, val as u64)
        }
    }
    #[inline]
    pub fn Ack(&self) -> u16 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(12usize, 1u8) as u16) }
    }
    #[inline]
    pub fn set_Ack(&mut self, val: u16) {
        unsafe {
            let val: u16 = ::std::mem::transmute(val);
            self.addr_bitfield.set(12usize, 1u8, val as u64)
        }
    }
    #[inline]
    pub fn Urg(&self) -> u16 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(13usize, 1u8) as u16) }
    }
    #[inline]
    pub fn set_Urg(&mut self, val: u16) {
        unsafe {
            let val: u16 = ::std::mem::transmute(val);
            self.addr_bitfield.set(13usize, 1u8, val as u64)
        }
    }
    #[inline]
    pub fn reserved(&self) -> u16 {
        unsafe { ::std::mem::transmute(self.addr_bitfield.get(14usize, 2u8) as u16) }
    }
    #[inline]
    pub fn set_reserved(&mut self, val: u16) {
        unsafe {
            let val: u16 = ::std::mem::transmute(val);
            self.addr_bitfield.set(14usize, 2u8, val as u64)
        }
    }
    #[inline]
    pub fn newaddr_bitfield(
        Reserved1: u16,
        HdrLength: u16,
        Fin: u16,
        Syn: u16,
        Rst: u16,
        Psh: u16,
        Ack: u16,
        Urg: u16,
        reserved: u16,
    ) -> BitfieldUnit<[u8; 2usize], u8> {
        let mut bitfield_unit: BitfieldUnit<[u8; 2usize], u8> = Default::default();
        bitfield_unit.set(0usize, 4u8, {
            let Reserved1: u16 = unsafe { ::std::mem::transmute(Reserved1) };
            Reserved1 as u64
        });
        bitfield_unit.set(4usize, 4u8, {
            let HdrLength: u16 = unsafe { ::std::mem::transmute(HdrLength) };
            HdrLength as u64
        });
        bitfield_unit.set(8usize, 1u8, {
            let Fin: u16 = unsafe { ::std::mem::transmute(Fin) };
            Fin as u64
        });
        bitfield_unit.set(9usize, 1u8, {
            let Syn: u16 = unsafe { ::std::mem::transmute(Syn) };
            Syn as u64
        });
        bitfield_unit.set(10usize, 1u8, {
            let Rst: u16 = unsafe { ::std::mem::transmute(Rst) };
            Rst as u64
        });
        bitfield_unit.set(11usize, 1u8, {
            let Psh: u16 = unsafe { ::std::mem::transmute(Psh) };
            Psh as u64
        });
        bitfield_unit.set(12usize, 1u8, {
            let Ack: u16 = unsafe { ::std::mem::transmute(Ack) };
            Ack as u64
        });
        bitfield_unit.set(13usize, 1u8, {
            let Urg: u16 = unsafe { ::std::mem::transmute(Urg) };
            Urg as u64
        });
        bitfield_unit.set(14usize, 2u8, {
            let reserved: u16 = unsafe { ::std::mem::transmute(reserved) };
            reserved as u64
        });
        bitfield_unit
    }
}
/// [TCP header](WINDIVERT_TCPHDR) pointer type.
pub type PWINDIVERT_TCPHDR = *mut WINDIVERT_TCPHDR;

/**
UDP header.

For more info, refer to the [docs](https://reqrypt.org/windivert-doc.html#divert_udphdr)
*/
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct WINDIVERT_UDPHDR {
    pub SrcPort: u16,
    pub DstPort: u16,
    pub Length: u16,
    pub Checksum: u16,
}

/// [UDP header](WINDIVERT_UDPHDR) pointer type.
pub type PWINDIVERT_UDPHDR = *mut WINDIVERT_UDPHDR;
