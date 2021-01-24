use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum WinDivertError {
    LayerValue,
    ParameterValue,
    ShutdownValue,
}

impl Display for WinDivertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WinDivertError::LayerValue => {
                write!(f, "Value doesn't represent a valid WinDivertLayer")
            }
            WinDivertError::ParameterValue => {
                write!(f, "Value doesn't represent a valid WinDivertPrameter")
            }
            WinDivertError::ShutdownValue => {
                write!(f, "Value doesn't represent a valid WinDivertShutdownValue")
            }
        }
    }
}

impl Error for WinDivertError {}
