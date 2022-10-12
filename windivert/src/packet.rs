use crate::{address::WinDivertAddress, layer};

use std::{borrow::Cow, fmt::Debug};

/// Raw packet using an already allocated buffer
#[derive(Debug, Clone)]
pub struct WinDivertPacket<'a, L: layer::WinDivertLayerTrait> {
    /// Address data
    pub address: WinDivertAddress<L>,
    /// Raw captured data
    pub data: Cow<'a, [u8]>,
}

impl<'a> WinDivertPacket<'a, layer::NetworkLayer> {
    /// Create a new network packet from a raw buffer
    /// SAFETY: `address` is zeroed, user must fill it with correct data before sending.
    pub unsafe fn new(data: Vec<u8>) -> Self {
        Self {
            address: WinDivertAddress::<layer::NetworkLayer>::new(),
            data: Cow::from(data),
        }
    }
}

impl<'a> WinDivertPacket<'a, layer::ForwardLayer> {
    /// Create a new network forward packet from a raw buffer
    /// SAFETY: `address` is zeroed`, user must fill it with correct data before sending.
    pub unsafe fn new(data: Vec<u8>) -> Self {
        Self {
            address: WinDivertAddress::<layer::ForwardLayer>::new(),
            data: Cow::from(data),
        }
    }
}

impl<'a, L: layer::WinDivertLayerTrait> WinDivertPacket<'a, L> {
    /// Create an owned packet from a borrowed packet
    pub fn into_owned(self) -> WinDivertPacket<'static, L> {
        WinDivertPacket {
            address: self.address,
            data: self.data.into_owned().into(),
        }
    }
}
