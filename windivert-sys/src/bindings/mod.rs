#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod address;
pub mod header;
pub mod ioctl;

mod bitfield;
pub(crate) use bitfield::BitfieldUnit;
mod error;
pub use error::WinDivertError;
mod newtypes;
pub use newtypes::*;

use winapi::{
    shared::{
        minwindef::BOOL,
        ntdef::{HANDLE, PVOID},
    },
    um::minwinbase::LPOVERLAPPED,
};

/// Default value for queue length parameter.
pub const WINDIVERT_PARAM_QUEUE_LENGTH_DEFAULT: u64 = 4096;
/// Minimum valid value for queue length parameter.
pub const WINDIVERT_PARAM_QUEUE_LENGTH_MIN: u64 = 32;
/// Maximum valid value for queue length parameter.
pub const WINDIVERT_PARAM_QUEUE_LENGTH_MAX: u64 = 16384;
/// Default value for queue time parameter.
pub const WINDIVERT_PARAM_QUEUE_TIME_DEFAULT: u64 = 2000; /* 2s */
/// Minimum valid value for queue time parameter.
pub const WINDIVERT_PARAM_QUEUE_TIME_MIN: u64 = 100; /* 100ms */
/// Maximum valid value for queue time parameter.
pub const WINDIVERT_PARAM_QUEUE_TIME_MAX: u64 = 16000; /* 16s */
/// Default value for queue size parameter.
pub const WINDIVERT_PARAM_QUEUE_SIZE_DEFAULT: u64 = 4194304; /* 4MB */
/// Minimum valid value for queue size parameter.
pub const WINDIVERT_PARAM_QUEUE_SIZE_MIN: u64 = 65535; /* 64KB */
/// Maximum valid value for queue size parameter.
pub const WINDIVERT_PARAM_QUEUE_SIZE_MAX: u64 = 33554432; /* 32MB */

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

    pub fn WinDivertSetParam(handle: HANDLE, param: WinDivertParam, value: u64) -> BOOL;

    pub fn WinDivertGetParam(handle: HANDLE, param: WinDivertParam, pValue: *mut u64) -> BOOL;
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
        flags: ChecksumFlags,
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
