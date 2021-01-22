#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod address;
pub mod header;

mod bitfield;
pub(crate) use bitfield::BitfieldUnit;

use winapi::{
    shared::{
        minwindef::BOOL,
        ntdef::{HANDLE, PVOID},
    },
    um::minwinbase::LPOVERLAPPED,
};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum WinDivertLayer {
    Network = 0,
    Forward = 1,
    Flow = 2,
    Socket = 3,
    Reflect = 4,
}

impl From<WinDivertLayer> for u32 {
    fn from(value: WinDivertLayer) -> Self {
        match value {
            WinDivertLayer::Network => 0,
            WinDivertLayer::Forward => 1,
            WinDivertLayer::Flow => 2,
            WinDivertLayer::Socket => 3,
            WinDivertLayer::Reflect => 4,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum WinDivertShutdownMode {
    None = 0,
    Recv = 1,
    Send = 2,
    Both = 3,
}

impl From<WinDivertShutdownMode> for u32 {
    fn from(value: WinDivertShutdownMode) -> Self {
        match value {
            WinDivertShutdownMode::None => 0,
            WinDivertShutdownMode::Recv => 1,
            WinDivertShutdownMode::Send => 2,
            WinDivertShutdownMode::Both => 3,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum WinDivertParam {
    QueueLength = 0,
    QueueTime = 1,
    QueueSize = 2,
    VersionMajor = 3,
    VersionMinor = 4,
}

impl From<WinDivertParam> for u32 {
    fn from(value: WinDivertParam) -> Self {
        match value {
            WinDivertParam::QueueLength => 0,
            WinDivertParam::QueueTime => 1,
            WinDivertParam::QueueSize => 2,
            WinDivertParam::VersionMajor => 3,
            WinDivertParam::VersionMinor => 4,
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Default, Copy, Clone)]
pub struct WINDIVERT_IOCTL_RECV {
    pub addr: u64,
    pub addr_len_ptr: u64,
}

extern "C" {
    pub fn WinDivertOpen(
        filter: *const ::std::os::raw::c_char,
        layer: u32,
        priority: i16,
        flags: u64,
    ) -> HANDLE;

    pub fn WinDivertRecv(
        handle: HANDLE,
        pPacket: *mut ::std::os::raw::c_void,
        packetLen: u32,
        pRecvLen: *mut u32,
        pAddr: *mut address::WINDIVERT_ADDRESS,
    ) -> BOOL;

    pub fn WinDivertRecvEx(
        handle: HANDLE,
        pPacket: *mut ::std::os::raw::c_void,
        packetLen: u32,
        pRecvLen: *mut u32,
        flags: u64,
        pAddr: *mut address::WINDIVERT_ADDRESS,
        pAddrLen: *mut u32,
        lpOverlapped: LPOVERLAPPED,
    ) -> BOOL;

    pub fn WinDivertSend(
        handle: HANDLE,
        pPacket: *const ::std::os::raw::c_void,
        packetLen: u32,
        pSendLen: *mut u32,
        pAddr: *const address::WINDIVERT_ADDRESS,
    ) -> BOOL;

    pub fn WinDivertSendEx(
        handle: HANDLE,
        pPacket: *const ::std::os::raw::c_void,
        packetLen: u32,
        pSendLen: *mut u32,
        flags: u64,
        pAddr: *const address::WINDIVERT_ADDRESS,
        addrLen: u32,
        lpOverlapped: LPOVERLAPPED,
    ) -> BOOL;

    pub fn WinDivertShutdown(handle: HANDLE, how: u32) -> BOOL;

    pub fn WinDivertClose(handle: HANDLE) -> BOOL;

    pub fn WinDivertSetParam(handle: HANDLE, param: u32, value: u64) -> BOOL;

    pub fn WinDivertGetParam(handle: HANDLE, param: u32, pValue: *mut u64) -> BOOL;
}

extern "C" {
    pub fn WinDivertHelperHashPacket(
        pPacket: *const ::std::os::raw::c_void,
        packetLen: u32,
        seed: u64,
    ) -> u64;

    pub fn WinDivertHelperParsePacket(
        pPacket: *const ::std::os::raw::c_void,
        packetLen: u32,
        ppIpHdr: *mut header::PWINDIVERT_IPHDR,
        ppIpv6Hdr: *mut header::PWINDIVERT_IPV6HDR,
        pProtocol: *mut u8,
        ppIcmpHdr: *mut header::PWINDIVERT_ICMPHDR,
        ppIcmpv6Hdr: *mut header::PWINDIVERT_ICMPV6HDR,
        ppTcpHdr: *mut header::PWINDIVERT_TCPHDR,
        ppUdpHdr: *mut header::PWINDIVERT_UDPHDR,
        ppData: *mut PVOID,
        pDataLen: *mut u32,
        ppNext: *mut PVOID,
        pNextLen: *mut u32,
    ) -> BOOL;

    pub fn WinDivertHelperParseIPv4Address(
        addrStr: *const ::std::os::raw::c_char,
        pAddr: *mut u32,
    ) -> BOOL;

    pub fn WinDivertHelperParseIPv6Address(
        addrStr: *const ::std::os::raw::c_char,
        pAddr: *mut u32,
    ) -> BOOL;

    pub fn WinDivertHelperFormatIPv4Address(
        addr: u32,
        buffer: *mut ::std::os::raw::c_char,
        bufLen: u32,
    ) -> BOOL;

    pub fn WinDivertHelperFormatIPv6Address(
        pAddr: *const u32,
        buffer: *mut ::std::os::raw::c_char,
        bufLen: u32,
    ) -> BOOL;

    pub fn WinDivertHelperCalcChecksums(
        pPacket: *mut ::std::os::raw::c_void,
        packetLen: u32,
        pAddr: *mut address::WINDIVERT_ADDRESS,
        flags: u64,
    ) -> BOOL;

    pub fn WinDivertHelperDecrementTTL(
        pPacket: *mut ::std::os::raw::c_void,
        packetLen: u32,
    ) -> BOOL;

    pub fn WinDivertHelperCompileFilter(
        filter: *const ::std::os::raw::c_char,
        layer: u32,
        object: *mut ::std::os::raw::c_char,
        objLen: u32,
        errorStr: *mut *const ::std::os::raw::c_char,
        errorPos: *mut u32,
    ) -> BOOL;

    pub fn WinDivertHelperEvalFilter(
        filter: *const ::std::os::raw::c_char,
        pPacket: *const ::std::os::raw::c_void,
        packetLen: u32,
        pAddr: *const address::WINDIVERT_ADDRESS,
    ) -> BOOL;

    pub fn WinDivertHelperFormatFilter(
        filter: *const ::std::os::raw::c_char,
        layer: u32,
        buffer: *mut ::std::os::raw::c_char,
        bufLen: u32,
    ) -> BOOL;

    pub fn WinDivertHelperNtohs(x: u16) -> u16;

    pub fn WinDivertHelperHtons(x: u16) -> u16;

    pub fn WinDivertHelperNtohl(x: u32) -> u32;

    pub fn WinDivertHelperHtonl(x: u32) -> u32;

    pub fn WinDivertHelperNtohll(x: u64) -> u64;

    pub fn WinDivertHelperHtonll(x: u64) -> u64;

    pub fn WinDivertHelperNtohIPv6Address(inAddr: *const u32, outAddr: *mut u32);

    pub fn WinDivertHelperHtonIPv6Address(inAddr: *const u32, outAddr: *mut u32);

    pub fn WinDivertHelperNtohIpv6Address(inAddr: *const u32, outAddr: *mut u32);

    pub fn WinDivertHelperHtonIpv6Address(inAddr: *const u32, outAddr: *mut u32);
}
