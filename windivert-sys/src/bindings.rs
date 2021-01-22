#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use winapi::shared::minwindef::BOOL;
use winapi::um::minwinbase::LPOVERLAPPED;
use winapi::um::winnt::{HANDLE, PVOID};

include!(concat!(env!("OUT_DIR"), "/generated_bindings.rs"));

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct WINDIVERT_ICMPHDR {
    pub Type: u8,
    pub Code: u8,
    pub Checksum: u16,
    pub Body: u32,
}

pub type PWINDIVERT_ICMPHDR = *mut WINDIVERT_ICMPHDR;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct WINDIVERT_ICMPV6HDR {
    pub Type: u8,
    pub Code: u8,
    pub Checksum: u16,
    pub Body: u32,
}

pub type PWINDIVERT_ICMPV6HDR = *mut WINDIVERT_ICMPV6HDR;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct WINDIVERT_UDPHDR {
    pub SrcPort: u16,
    pub DstPort: u16,
    pub Length: u16,
    pub Checksum: u16,
}

pub type PWINDIVERT_UDPHDR = *mut WINDIVERT_UDPHDR;

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
        pAddr: *mut WINDIVERT_ADDRESS,
    ) -> BOOL;

    pub fn WinDivertRecvEx(
        handle: HANDLE,
        pPacket: *mut ::std::os::raw::c_void,
        packetLen: u32,
        pRecvLen: *mut u32,
        flags: u64,
        pAddr: *mut WINDIVERT_ADDRESS,
        pAddrLen: *mut u32,
        lpOverlapped: LPOVERLAPPED,
    ) -> BOOL;

    pub fn WinDivertSend(
        handle: HANDLE,
        pPacket: *const ::std::os::raw::c_void,
        packetLen: u32,
        pSendLen: *mut u32,
        pAddr: *const WINDIVERT_ADDRESS,
    ) -> BOOL;

    pub fn WinDivertSendEx(
        handle: HANDLE,
        pPacket: *const ::std::os::raw::c_void,
        packetLen: u32,
        pSendLen: *mut u32,
        flags: u64,
        pAddr: *const WINDIVERT_ADDRESS,
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
        ppIpHdr: *mut PWINDIVERT_IPHDR,
        ppIpv6Hdr: *mut PWINDIVERT_IPV6HDR,
        pProtocol: *mut u8,
        ppIcmpHdr: *mut PWINDIVERT_ICMPHDR,
        ppIcmpv6Hdr: *mut PWINDIVERT_ICMPV6HDR,
        ppTcpHdr: *mut PWINDIVERT_TCPHDR,
        ppUdpHdr: *mut PWINDIVERT_UDPHDR,
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
        pAddr: *mut WINDIVERT_ADDRESS,
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
        pAddr: *const WINDIVERT_ADDRESS,
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
