/// Network type for typestate pattern.
#[derive(Debug, Clone)]
pub enum NetworkLayer {}
/// Forward type for typestate pattern.
#[derive(Debug, Clone)]
pub enum ForwardLayer {}
/// Flow type for typestate pattern.
#[derive(Debug, Clone)]
pub enum FlowLayer {}
/// Socket type for typestate pattern.
#[derive(Debug, Clone)]
pub enum SocketLayer {}
/// Reflect type for typestate pattern.
#[derive(Debug, Clone)]
pub enum ReflectLayer {}

/// Trait for typestate pattern.
pub trait WinDivertLayerTrait: sealed::Sealed + std::fmt::Debug + std::clone::Clone {
    /// Utility method to identify data capturing layers in the typestate
    fn captures_data() -> bool {
        false
    }
}

impl WinDivertLayerTrait for NetworkLayer {
    fn captures_data() -> bool {
        true
    }
}

impl WinDivertLayerTrait for ForwardLayer {
    fn captures_data() -> bool {
        true
    }
}

impl WinDivertLayerTrait for FlowLayer {}

impl WinDivertLayerTrait for SocketLayer {}

impl WinDivertLayerTrait for ReflectLayer {}

impl WinDivertLayerTrait for () {}

mod sealed {
    pub trait Sealed {}

    impl Sealed for () {}
    impl Sealed for super::NetworkLayer {}
    impl Sealed for super::ForwardLayer {}
    impl Sealed for super::FlowLayer {}
    impl Sealed for super::SocketLayer {}
    impl Sealed for super::ReflectLayer {}
}
