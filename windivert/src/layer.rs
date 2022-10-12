use windivert_sys::WinDivertLayer;

pub enum NetworkLayer {}
pub enum ForwardLayer {}
pub enum FlowLayer {}
pub enum SocketLayer {}
pub enum ReflectLayer {}

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
