use std::error::Error;
use std::fmt::Display;

/**
WinDivert error for unexpected values type conversions.
*/
#[derive(Debug)]
pub enum WinDivertValueError {
    Layer,
    Event,
    Parameter,
    Shutdown,
}

impl Display for WinDivertValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for WinDivertValueError {}
