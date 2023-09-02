use crate::{address::WinDivertAddress, layer};

use std::borrow::Cow;

/// Raw partial captured packet
#[derive(Debug, Clone)]
pub struct WinDivertPartialPacket<'a, L: layer::WinDivertLayerTrait> {
    /// Address data
    pub address: WinDivertAddress<L>,
    /// Raw captured data
    pub data: Cow<'a, [u8]>,
}
