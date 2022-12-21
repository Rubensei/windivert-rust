use windivert_sys::WinDivertLayer;

/// Network type for typestate pattern.
pub enum NetworkLayer {}
/// Forward type for typestate pattern.
pub enum ForwardLayer {}
/// Flow type for typestate pattern.
pub enum FlowLayer {}
/// Socket type for typestate pattern.
pub enum SocketLayer {}
/// Reflect type for typestate pattern.
pub enum ReflectLayer {}

/// Trait for typestate pattern.
pub trait WinDivertLayerTrait: sealed::Sealed {}

impl WinDivertLayerTrait for NetworkLayer {}

impl WinDivertLayerTrait for ForwardLayer {}

impl WinDivertLayerTrait for FlowLayer {}

impl WinDivertLayerTrait for SocketLayer {}

impl WinDivertLayerTrait for ReflectLayer {}

impl WinDivertLayerTrait for WinDivertLayer {}

mod sealed {
    pub trait Sealed {}

    impl Sealed for super::NetworkLayer {}
    impl Sealed for super::ForwardLayer {}
    impl Sealed for super::FlowLayer {}
    impl Sealed for super::SocketLayer {}
    impl Sealed for super::ReflectLayer {}
    impl Sealed for super::WinDivertLayer {}
}
