use std::convert::TryFrom;
use std::ffi::NulError;

use thiserror::Error;
use windivert_sys::{WinDivertParam, WinDivertValueError};
use windows::Win32::Foundation::WIN32_ERROR;

/**
WinDivert error type.
*/
#[derive(Debug, Error)]
pub enum WinDivertError {
    /// Unexpected value in type conversions.
    #[error(transparent)]
    Value(#[from] WinDivertValueError),
    /// Specific errors for divert constructor invocation.
    #[error(transparent)]
    Open(#[from] WinDivertOpenError),
    /// Specific errors for `WinDivert::recv()`.
    #[error(transparent)]
    Recv(#[from] WinDivertRecvError),
    /// Specific errors for `WinDivert::send()`.
    #[error(transparent)]
    Send(#[from] WinDivertSendError),
    /// Error for nul terminated filter strings.
    #[error(transparent)]
    NullError(#[from] NulError),
    /// Generic OS error.
    #[error(transparent)]
    OSError(#[from] windows::core::Error),
    /// Error indicating that a wrong parameter was used in `WinDivert::set_param()`
    #[error("Invalid parameter for set_param(). Parameter: {0:?}, Value: {1}")]
    Parameter(WinDivertParam, u64),
    /// Timeout error.
    #[error("Wait operation timed out")]
    Timeout,
}

/// Possible errors for [`WinDivertOpen()`](fn@windivert_sys::WinDivertOpen)
#[derive(Debug, Error)]
pub enum WinDivertOpenError {
    /// The driver files WinDivert32.sys or WinDivert64.sys were not found.
    #[error("SYS driver file not found")]
    MissingSYS, // 2
    /// The calling application does not have Administrator privileges.
    #[error("Running without elevated access rights")]
    AccessDenied, // 5
    /// This indicates an invalid packet filter string, layer, priority, or flags.
    #[error("Invalid parameter (filter string, layer, priority, or flags)")]
    InvalidParameter, // 87
    /// The WinDivert32.sys or WinDivert64.sys driver does not have a valid digital signature.
    #[error("SYS driver file has invalid digital signature")]
    InvalidImageHash, // 577
    /// An incompatible version of the WinDivert driver is currently loaded.
    #[error("An incompatible version of the WinDivert driver is currently loaded")]
    IncompatibleVersion, // 654
    /// The handle was opened with the WINDIVERT_FLAG_NO_INSTALL flag and the WinDivert driver is not already installed.
    #[error("The handle was opened with the WINDIVERT_FLAG_NO_INSTALL flag and the WinDivert driver is not already installed")]
    MissingInstall, // 1060
    /// The WinDivert driver is blocked by security software or you are using a virtualization environment that does not support drivers.
    #[error("WinDivert driver is blocked by security software or you are using a virtualization environment that does not support drivers")]
    DriverBlocked, // 1257
    /// This error occurs when the Base Filtering Engine service has been disabled.
    #[error("Base Filtering Engine service has been disabled")]
    BaseFilteringEngineDisabled, // 1753
}

impl TryFrom<windows::core::Error> for WinDivertOpenError {
    type Error = windows::core::Error;

    fn try_from(error: windows::core::Error) -> Result<Self, Self::Error> {
        let Some(win32_error) = WIN32_ERROR::from_error(&error) else {
            return Err(error);
        };
        match win32_error {
            WIN32_ERROR(2u32) => Ok(WinDivertOpenError::MissingSYS),
            WIN32_ERROR(5u32) => Ok(WinDivertOpenError::AccessDenied),
            WIN32_ERROR(87u32) => Ok(WinDivertOpenError::InvalidParameter),
            WIN32_ERROR(577u32) => Ok(WinDivertOpenError::InvalidImageHash),
            WIN32_ERROR(654u32) => Ok(WinDivertOpenError::IncompatibleVersion),
            WIN32_ERROR(1060u32) => Ok(WinDivertOpenError::MissingInstall),
            WIN32_ERROR(1257u32) => Ok(WinDivertOpenError::DriverBlocked),
            WIN32_ERROR(1753u32) => Ok(WinDivertOpenError::BaseFilteringEngineDisabled),
            _ => Err(error),
        }
    }
}

impl TryFrom<WIN32_ERROR> for WinDivertOpenError {
    type Error = windows::core::Error;

    fn try_from(error: WIN32_ERROR) -> Result<Self, Self::Error> {
        WinDivertOpenError::try_from(windows::core::Error::from_hresult(error.to_hresult()))
    }
}

/// Possible errors for [`WinDivertRecv()`](fn@windivert_sys::WinDivertRecv)
#[derive(Debug, Error)]
pub enum WinDivertRecvError {
    /// The captured packet is larger than the provided buffer.
    #[error("Captured packet is larger than the provided buffer")]
    InsufficientBuffer, // 122
    /// The handle has been shutdown and the packet queue is empty.
    #[error("Not possible to get more data. Packet queue is empty and handle has been shutdown")]
    NoData, // 232
}

impl TryFrom<windows::core::Error> for WinDivertRecvError {
    type Error = windows::core::Error;

    fn try_from(error: windows::core::Error) -> Result<Self, Self::Error> {
        let Some(win32_error) = WIN32_ERROR::from_error(&error) else {
            return Err(error);
        };
        match win32_error {
            WIN32_ERROR(122u32) => Ok(WinDivertRecvError::InsufficientBuffer),
            WIN32_ERROR(232u32) => Ok(WinDivertRecvError::NoData),
            _ => Err(error),
        }
    }
}

impl TryFrom<WIN32_ERROR> for WinDivertRecvError {
    type Error = windows::core::Error;

    fn try_from(error: WIN32_ERROR) -> Result<Self, Self::Error> {
        WinDivertRecvError::try_from(windows::core::Error::from_hresult(error.to_hresult()))
    }
}

/// Possible errors for `WinDivert::send()` methods.
#[derive(Debug, Error)]
pub enum WinDivertSendError {
    /// WinDivert can't send more than [`WINDIVERT_BATCH_MAX`](windivert_sys::WINDIVERT_BATCH_MAX) packets at once.
    #[error("Provided packet slice is too large")]
    TooManyPackets,
    /// WinDivert will return this error if the TTL of an _impostor_ packet reaches 0.
    #[error("Host unreachable")]
    HostUnrachable, // 1232
}

impl TryFrom<windows::core::Error> for WinDivertSendError {
    type Error = windows::core::Error;

    fn try_from(error: windows::core::Error) -> Result<Self, Self::Error> {
        let Some(win32_error) = WIN32_ERROR::from_error(&error) else {
            return Err(error);
        };
        match win32_error {
            WIN32_ERROR(1232u32) => Ok(WinDivertSendError::HostUnrachable),
            _ => Err(error),
        }
    }
}

impl TryFrom<WIN32_ERROR> for WinDivertSendError {
    type Error = windows::core::Error;

    fn try_from(error: WIN32_ERROR) -> Result<Self, Self::Error> {
        WinDivertSendError::try_from(windows::core::Error::from_hresult(error.to_hresult()))
    }
}
