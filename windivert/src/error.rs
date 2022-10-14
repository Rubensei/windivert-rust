use std::{convert::TryFrom, error::Error};
use std::{ffi::NulError, fmt::Display};

use windivert_sys::WinDivertValueError;

/**
WinDivert error type.
*/
#[derive(Debug)]
pub enum WinDivertError {
    /// Unexpected value in type conversions.
    Value(WinDivertValueError),
    /// Specific errors for divert constructor invocation.
    Open(WinDivertOpenError),
    /// Specific errors for [`WinDivert::recv()`](fn@super::WinDivert::<L>::recv).
    Recv(WinDivertRecvError),
    /// Error for nul terminated filter strings.
    NullError(NulError),
    /// Generic OS error.
    OSError(std::io::Error),
    /// Error indicating that a wrong parameter was used in [`set_param()`](fn@crate::WinDivert::set_param)
    Parameter,
}

impl From<WinDivertValueError> for WinDivertError {
    fn from(error: WinDivertValueError) -> Self {
        Self::Value(error)
    }
}

impl From<WinDivertOpenError> for WinDivertError {
    fn from(error: WinDivertOpenError) -> Self {
        Self::Open(error)
    }
}

impl From<WinDivertRecvError> for WinDivertError {
    fn from(error: WinDivertRecvError) -> Self {
        Self::Recv(error)
    }
}

impl From<windows::core::Error> for WinDivertError {
    fn from(error: windows::core::Error) -> Self {
        Self::OSError(std::io::Error::from_raw_os_error(error.code().0))
    }
}

/**
Possible errors for [`WinDivertOpen()`](fn@windivert_sys::WinDivertOpen)
*/
#[derive(Debug)]
pub enum WinDivertOpenError {
    /// The driver files WinDivert32.sys or WinDivert64.sys were not found.
    MissingSYS, // 2
    /// The calling application does not have Administrator privileges.
    AccessDenied, // 5
    /// This indicates an invalid packet filter string, layer, priority, or flags.
    InvalidParameter, // 87
    /// The WinDivert32.sys or WinDivert64.sys driver does not have a valid digital signature.
    InvalidImageHash, // 577
    /// An incompatible version of the WinDivert driver is currently loaded.
    IncompatibleVersion, // 654
    /// The handle was opened with the WINDIVERT_FLAG_NO_INSTALL flag and the WinDivert driver is not already installed.
    MissingInstall, // 1060
    /// The WinDivert driver is blocked by security software or you are using a virtualization environment that does not support drivers.
    DriverBlocked, // 1257
    /// This error occurs when the Base Filtering Engine service has been disabled.
    BaseFilteringEngineDisabled, // 1753
}

impl TryFrom<i32> for WinDivertOpenError {
    type Error = std::io::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            2 => Ok(WinDivertOpenError::MissingSYS),
            5 => Ok(WinDivertOpenError::AccessDenied),
            87 => Ok(WinDivertOpenError::InvalidParameter),
            577 => Ok(WinDivertOpenError::InvalidImageHash),
            654 => Ok(WinDivertOpenError::IncompatibleVersion),
            1060 => Ok(WinDivertOpenError::MissingInstall),
            1257 => Ok(WinDivertOpenError::DriverBlocked),
            1753 => Ok(WinDivertOpenError::BaseFilteringEngineDisabled),
            _ => Err(std::io::Error::from_raw_os_error(value)),
        }
    }
}

impl TryFrom<std::io::Error> for WinDivertOpenError {
    type Error = std::io::Error;

    fn try_from(error: std::io::Error) -> Result<Self, Self::Error> {
        if let Some(value) = error.raw_os_error() {
            match value {
                2 => Ok(WinDivertOpenError::MissingSYS),
                5 => Ok(WinDivertOpenError::AccessDenied),
                87 => Ok(WinDivertOpenError::InvalidParameter),
                577 => Ok(WinDivertOpenError::InvalidImageHash),
                654 => Ok(WinDivertOpenError::IncompatibleVersion),
                1060 => Ok(WinDivertOpenError::MissingInstall),
                1257 => Ok(WinDivertOpenError::DriverBlocked),
                1753 => Ok(WinDivertOpenError::BaseFilteringEngineDisabled),
                _ => Err(std::io::Error::from_raw_os_error(value)),
            }
        } else {
            Err(error)
        }
    }
}

/**
Possible errors for [`WinDivertRecv()`](fn@windivert_sys::WinDivertRecv)
*/
#[derive(Debug)]
pub enum WinDivertRecvError {
    /// The captured packet is larger than the provided buffer.
    InsufficientBuffer, // 122
    /// The handle has been shutdown and the packet queue is empty.
    NoData, // 232
}

impl TryFrom<i32> for WinDivertRecvError {
    type Error = std::io::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            122 => Ok(WinDivertRecvError::InsufficientBuffer),
            232 => Ok(WinDivertRecvError::NoData),
            _ => Err(std::io::Error::from_raw_os_error(value)),
        }
    }
}

impl TryFrom<std::io::Error> for WinDivertRecvError {
    type Error = std::io::Error;

    fn try_from(error: std::io::Error) -> Result<Self, Self::Error> {
        if let Some(value) = error.raw_os_error() {
            match value {
                122 => Ok(WinDivertRecvError::InsufficientBuffer),
                232 => Ok(WinDivertRecvError::NoData),
                _ => Err(std::io::Error::from_raw_os_error(value)),
            }
        } else {
            Err(error)
        }
    }
}

impl From<NulError> for WinDivertError {
    fn from(e: NulError) -> Self {
        WinDivertError::NullError(e)
    }
}

impl Into<WinDivertError> for std::io::Error {
    fn into(self) -> WinDivertError {
        WinDivertError::OSError(self)
    }
}

impl Display for WinDivertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for WinDivertError {}
