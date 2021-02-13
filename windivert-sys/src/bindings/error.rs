use std::error::Error;
use std::fmt::Display;

/**
WinDivert error for unexpected values type conversions.
*/
#[derive(Debug)]
pub enum WinDivertValueError {
    /// Error produced for unexpected values in TryFrom<u32> for [`WinDivertLayer`](super::WinDivertLayer)
    Layer,
    /// Error produced for unexpected values in TryFrom<u8> for [`WinDivertEvent`](super::WinDivertEvent)
    Event,
    /// Error produced for unexpected values in TryFrom<u32> for [`WinDivertParameter`](super::WinDivertParam)
    Parameter,
    /// Error produced for unexpected values in TryFrom<u32> for [`WinDivertShutdownMode`](super::WinDivertShutdownMode)
    Shutdown,
}

impl Display for WinDivertValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for WinDivertValueError {}
