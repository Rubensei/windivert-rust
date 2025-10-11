#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod address;
pub mod header;
pub mod ioctl;

mod bitfield;

pub(crate) use bitfield::BitfieldUnit;
mod error;
pub use error::*;
mod newtypes;
pub use newtypes::*;

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
/// Maximum valid value for priority parameter.
pub const WINDIVERT_PRIORITY_MAX: i16 = 30000;
/// Minimum valid value for priority parameter.
pub const WINDIVERT_PRIORITY_MIN: i16 = -30000;
/// Maximum valid batch length.
pub const WINDIVERT_BATCH_MAX: u32 = 255;
/// Maximum valid mtu size.
pub const WINDIVERT_MTU_MAX: u32 = 65575;

/// Alias type for meaningful retur types
pub type BOOL = core::ffi::c_int;

extern "C" {
    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_open)
    pub fn WinDivertOpen(
        filter: *const core::ffi::c_char,
        layer: WinDivertLayer,
        priority: i16,
        flags: WinDivertFlags,
    ) -> std::os::windows::raw::HANDLE;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_recv)
    pub fn WinDivertRecv(
        handle: *mut core::ffi::c_void,
        pPacket: *mut core::ffi::c_void,
        packetLen: u32,
        pRecvLen: *mut u32,
        pAddr: *mut address::WINDIVERT_ADDRESS,
    ) -> BOOL;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_recv_ex)
    pub fn WinDivertRecvEx(
        handle: *mut core::ffi::c_void,
        pPacket: *mut core::ffi::c_void,
        packetLen: u32,
        pRecvLen: *mut u32,
        flags: u64,
        pAddr: *mut address::WINDIVERT_ADDRESS,
        pAddrLen: *mut u32,
        lpOverlapped: *mut core::ffi::c_void,
    ) -> BOOL;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_send)
    pub fn WinDivertSend(
        handle: *mut core::ffi::c_void,
        pPacket: *const core::ffi::c_void,
        packetLen: u32,
        pSendLen: *mut u32,
        pAddr: *const address::WINDIVERT_ADDRESS,
    ) -> BOOL;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_send_ex)
    pub fn WinDivertSendEx(
        handle: *mut core::ffi::c_void,
        pPacket: *const core::ffi::c_void,
        packetLen: u32,
        pSendLen: *mut u32,
        flags: u64,
        pAddr: *const address::WINDIVERT_ADDRESS,
        addrLen: u32,
        lpOverlapped: *mut core::ffi::c_void,
    ) -> BOOL;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_shutdown)
    pub fn WinDivertShutdown(handle: *mut core::ffi::c_void, how: WinDivertShutdownMode) -> BOOL;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_close)
    pub fn WinDivertClose(handle: *mut core::ffi::c_void) -> BOOL;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_set_param)
    pub fn WinDivertSetParam(
        handle: *mut core::ffi::c_void,
        param: WinDivertParam,
        value: u64,
    ) -> BOOL;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_get_param)
    pub fn WinDivertGetParam(
        handle: *mut core::ffi::c_void,
        param: WinDivertParam,
        pValue: *mut u64,
    ) -> BOOL;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_helper_parse_packet)
    pub fn WinDivertHelperParsePacket(
        pPacket: *const core::ffi::c_void,
        packetLen: u32,
        ppIpHdr: *mut header::PWINDIVERT_IPHDR,
        ppIpv6Hdr: *mut header::PWINDIVERT_IPV6HDR,
        pProtocol: *mut u8,
        ppIcmpHdr: *mut header::PWINDIVERT_ICMPHDR,
        ppIcmpv6Hdr: *mut header::PWINDIVERT_ICMPV6HDR,
        ppTcpHdr: *mut header::PWINDIVERT_TCPHDR,
        ppUdpHdr: *mut header::PWINDIVERT_UDPHDR,
        ppData: *mut *mut core::ffi::c_void,
        pDataLen: *mut u32,
        ppNext: *mut *mut core::ffi::c_void,
        pNextLen: *mut u32,
    ) -> BOOL;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_helper_hash_packet)
    pub fn WinDivertHelperHashPacket(
        pPacket: *const core::ffi::c_void,
        packetLen: u32,
        seed: u64,
    ) -> u64;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_helper_parse_ipv4_address)
    pub fn WinDivertHelperParseIPv4Address(
        addrStr: *const core::ffi::c_char,
        pAddr: *mut u32,
    ) -> BOOL;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_helper_parse_ipv6_address)
    pub fn WinDivertHelperParseIPv6Address(
        addrStr: *const core::ffi::c_char,
        pAddr: *mut u32,
    ) -> BOOL;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_helper_format_ipv4_address)
    pub fn WinDivertHelperFormatIPv4Address(
        addr: u32,
        buffer: *mut core::ffi::c_char,
        bufLen: u32,
    ) -> BOOL;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_helper_format_ipv6_address)
    pub fn WinDivertHelperFormatIPv6Address(
        pAddr: *const u32,
        buffer: *mut core::ffi::c_char,
        bufLen: u32,
    ) -> BOOL;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_helper_calc_checksums)
    pub fn WinDivertHelperCalcChecksums(
        pPacket: *mut core::ffi::c_void,
        packetLen: u32,
        pAddr: *mut address::WINDIVERT_ADDRESS,
        flags: ChecksumFlags,
    ) -> BOOL;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_helper_dec_ttl)
    pub fn WinDivertHelperDecrementTTL(pPacket: *mut core::ffi::c_void, packetLen: u32) -> BOOL;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_helper_compile_filter)
    pub fn WinDivertHelperCompileFilter(
        filter: *const core::ffi::c_char,
        layer: WinDivertLayer,
        object: *mut core::ffi::c_char,
        objLen: u32,
        errorStr: *mut *const core::ffi::c_char,
        errorPos: *mut u32,
    ) -> BOOL;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_helper_eval_filter)
    pub fn WinDivertHelperEvalFilter(
        filter: *const core::ffi::c_char,
        pPacket: *const core::ffi::c_void,
        packetLen: u32,
        pAddr: *const address::WINDIVERT_ADDRESS,
    ) -> BOOL;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_helper_format_filter)
    pub fn WinDivertHelperFormatFilter(
        filter: *const core::ffi::c_char,
        layer: WinDivertLayer,
        buffer: *mut core::ffi::c_char,
        bufLen: u32,
    ) -> BOOL;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_helper_ntoh)
    pub fn WinDivertHelperNtohs(x: u16) -> u16;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_helper_hton)
    pub fn WinDivertHelperHtons(x: u16) -> u16;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_helper_ntoh)
    pub fn WinDivertHelperNtohl(x: u32) -> u32;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_helper_hton)
    pub fn WinDivertHelperHtonl(x: u32) -> u32;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_helper_ntoh)
    pub fn WinDivertHelperNtohll(x: u64) -> u64;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_helper_hton)
    pub fn WinDivertHelperHtonll(x: u64) -> u64;

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_helper_ntoh)
    pub fn WinDivertHelperNtohIPv6Address(inAddr: *const u32, outAddr: *mut u32);

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_helper_hton)
    pub fn WinDivertHelperHtonIPv6Address(inAddr: *const u32, outAddr: *mut u32);

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_helper_ntoh)
    pub fn WinDivertHelperNtohIpv6Address(inAddr: *const u32, outAddr: *mut u32);

    /// Check the official [docs](https://reqrypt.org/windivert-doc.html#divert_helper_hton)
    pub fn WinDivertHelperHtonIpv6Address(inAddr: *const u32, outAddr: *mut u32);
}
