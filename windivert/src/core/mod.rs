#![allow(non_snake_case)]
#![allow(unused)]

/// Winapi abstractions and safe wrappers around some winapi concepts and blocking calls
pub(crate) mod winapi;

#[cfg(test)]
use mockall::automock;

use windivert_sys::{address::WINDIVERT_ADDRESS, WinDivertFlags, WinDivertLayer};

#[derive(Default)]
pub(crate) struct SysWrapper;

#[cfg_attr(test, automock)]
impl SysWrapper {
    pub(crate) unsafe fn WinDivertOpen(
        &self,
        filter: *const core::ffi::c_char,
        layer: WinDivertLayer,
        priority: i16,
        flags: WinDivertFlags,
    ) -> *mut core::ffi::c_void {
        windivert_sys::WinDivertOpen(filter, layer, priority, flags)
    }

    pub(crate) unsafe fn WinDivertRecv(
        &self,
        handle: *mut core::ffi::c_void,
        pPacket: *mut core::ffi::c_void,
        packetLen: u32,
        pRecvLen: *mut u32,
        pAddr: *mut WINDIVERT_ADDRESS,
    ) -> i32 {
        windivert_sys::WinDivertRecv(handle, pPacket, packetLen, pRecvLen, pAddr)
    }

    pub(crate) unsafe fn WinDivertRecvEx(
        &self,
        handle: *mut core::ffi::c_void,
        pPacket: *mut core::ffi::c_void,
        packetLen: u32,
        pRecvLen: *mut u32,
        flags: u64,
        pAddr: *mut WINDIVERT_ADDRESS,
        pAddrLen: *mut u32,
        lpOverlapped: *mut core::ffi::c_void,
    ) -> i32 {
        windivert_sys::WinDivertRecvEx(
            handle,
            pPacket,
            packetLen,
            pRecvLen,
            flags,
            pAddr,
            pAddrLen,
            lpOverlapped,
        )
    }

    pub(crate) unsafe fn WinDivertSend(
        &self,
        handle: *mut core::ffi::c_void,
        pPacket: *const core::ffi::c_void,
        packetLen: u32,
        pSendLen: *mut u32,
        pAddr: *const WINDIVERT_ADDRESS,
    ) -> i32 {
        windivert_sys::WinDivertSend(handle, pPacket, packetLen, pSendLen, pAddr)
    }

    pub(crate) unsafe fn WinDivertSendEx(
        &self,
        handle: *mut core::ffi::c_void,
        pPacket: *const core::ffi::c_void,
        packetLen: u32,
        pSendLen: *mut u32,
        flags: u64,
        pAddr: *const WINDIVERT_ADDRESS,
        addrLen: u32,
        lpOverlapped: *mut core::ffi::c_void,
    ) -> i32 {
        windivert_sys::WinDivertSendEx(
            handle,
            pPacket,
            packetLen,
            pSendLen,
            flags,
            pAddr,
            addrLen,
            lpOverlapped,
        )
    }
}

#[cfg_attr(test, automock)]
pub(crate) mod divert_blocking_wrapper {
    #![allow(unused)]
    use windivert_sys::{address::WINDIVERT_ADDRESS, WinDivertFlags, WinDivertLayer};

    pub(crate) unsafe fn WinDivertOpen(
        filter: *const core::ffi::c_char,
        layer: WinDivertLayer,
        priority: i16,
        flags: WinDivertFlags,
    ) -> *mut core::ffi::c_void {
        windivert_sys::WinDivertOpen(filter, layer, priority, flags)
    }

    pub(crate) unsafe fn WinDivertRecv(
        handle: *mut core::ffi::c_void,
        pPacket: *mut core::ffi::c_void,
        packetLen: u32,
        pRecvLen: *mut u32,
        pAddr: *mut WINDIVERT_ADDRESS,
    ) -> i32 {
        windivert_sys::WinDivertRecv(handle, pPacket, packetLen, pRecvLen, pAddr)
    }

    pub(crate) unsafe fn WinDivertRecvEx(
        handle: *mut core::ffi::c_void,
        pPacket: *mut core::ffi::c_void,
        packetLen: u32,
        pRecvLen: *mut u32,
        flags: u64,
        pAddr: *mut WINDIVERT_ADDRESS,
        pAddrLen: *mut u32,
        lpOverlapped: *mut core::ffi::c_void,
    ) -> i32 {
        windivert_sys::WinDivertRecvEx(
            handle,
            pPacket,
            packetLen,
            pRecvLen,
            flags,
            pAddr,
            pAddrLen,
            lpOverlapped,
        )
    }

    pub(crate) unsafe fn WinDivertSend(
        handle: *mut core::ffi::c_void,
        pPacket: *const core::ffi::c_void,
        packetLen: u32,
        pSendLen: *mut u32,
        pAddr: *const WINDIVERT_ADDRESS,
    ) -> i32 {
        windivert_sys::WinDivertSend(handle, pPacket, packetLen, pSendLen, pAddr)
    }

    pub(crate) unsafe fn WinDivertSendEx(
        handle: *mut core::ffi::c_void,
        pPacket: *const core::ffi::c_void,
        packetLen: u32,
        pSendLen: *mut u32,
        flags: u64,
        pAddr: *const WINDIVERT_ADDRESS,
        addrLen: u32,
        lpOverlapped: *mut core::ffi::c_void,
    ) -> i32 {
        windivert_sys::WinDivertSendEx(
            handle,
            pPacket,
            packetLen,
            pSendLen,
            flags,
            pAddr,
            addrLen,
            lpOverlapped,
        )
    }
}
