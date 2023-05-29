use std::{convert::TryFrom, u32};

use super::WinDivertValueError;

/**
WinDivert layer to initialize the handle.

WinDivert supports several layers for diverting or capturing network packets/events. Each layer has its own capabilities, such as the ability to block events or to inject new events, etc. The list of supported WinDivert layers is summarized below:

| Layer     | Block?     | Inject?     | Data? | PID? | Description                                        |
| --------- | ---------- | ----------- | ----- | ---- | -------------------------------------------------- |
| `Network` | ✔          | ✔           | ✔     |      | Network packets to/from the local machine.         |
| `Forward` | ✔          | ✔           | ✔     |      | Network packets passing through the local machine. |
| `Flow`    |            |             |       | ✔    | Network flow established/deleted events.           |
| `Socket`  | ✔          |             |       | ✔    | Socket operation events.                           |
| `Reflect` |            |             | ✔     | ✔    | WinDivert handle events.                           |
*/
#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum WinDivertLayer {
    /// Network packets to/from the local machine.
    Network = 0,
    /// Network packets passing through the local machine.
    Forward = 1,
    /// Network flow established/deleted events.
    Flow = 2,
    /// Socket operation events
    Socket = 3,
    /// WinDivert handle events.
    Reflect = 4,
}

impl TryFrom<u32> for WinDivertLayer {
    type Error = WinDivertValueError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(WinDivertLayer::Network),
            1 => Ok(WinDivertLayer::Forward),
            2 => Ok(WinDivertLayer::Flow),
            3 => Ok(WinDivertLayer::Socket),
            4 => Ok(WinDivertLayer::Reflect),
            _ => Err(WinDivertValueError::Layer(value)),
        }
    }
}

impl From<WinDivertLayer> for u8 {
    fn from(value: WinDivertLayer) -> Self {
        match value {
            WinDivertLayer::Network => 0,
            WinDivertLayer::Forward => 1,
            WinDivertLayer::Flow => 2,
            WinDivertLayer::Socket => 3,
            WinDivertLayer::Reflect => 4,
        }
    }
}

impl From<WinDivertLayer> for u32 {
    fn from(value: WinDivertLayer) -> Self {
        u8::from(value) as u32
    }
}

/**
WinDivert event identifiers.

Each [`WinDivertLayer`] supports one or more kind of events:
 * [`Network`](WinDivertLayer::Network) and [`Forward`](WinDivertLayer::Forward):
   * [`WinDivertEvent::NetworkPacket`]
 * [`Flow`](WinDivertLayer::Flow):
   * [`WinDivertEvent::FlowStablished`]
   * [`WinDivertEvent::FlowDeleted`]
 * [`Socket`](WinDivertLayer::Socket):
   * [`WinDivertEvent::SocketBind`]
   * [`WinDivertEvent::SocketConnect`]
   * [`WinDivertEvent::SocketListen`]
   * [`WinDivertEvent::SocketAccept`]
   * [`WinDivertEvent::SocketClose`]
 * [`Reflect`](WinDivertLayer::Reflect):
   * [`WinDivertEvent::ReflectOpen`]
   * [`WinDivertEvent::ReflectClose`]
*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum WinDivertEvent {
    /// Network packet.
    NetworkPacket = 0,
    /// Flow established.
    FlowStablished = 1,
    /// Flow deleted.
    FlowDeleted = 2,
    /// Socket bind.
    SocketBind = 3,
    /// Socket connect.
    SocketConnect = 4,
    /// Socket listen.
    SocketListen = 5,
    /// Socket accept.
    SocketAccept = 6,
    /// Socket close.
    SocketClose = 7,
    /// WinDivert handle opened.
    ReflectOpen = 8,
    /// WinDivert handle closed.
    ReflectClose = 9,
}

impl TryFrom<u8> for WinDivertEvent {
    type Error = WinDivertValueError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NetworkPacket),
            1 => Ok(Self::FlowStablished),
            2 => Ok(Self::FlowDeleted),
            3 => Ok(Self::SocketBind),
            4 => Ok(Self::SocketConnect),
            5 => Ok(Self::SocketListen),
            6 => Ok(Self::SocketAccept),
            7 => Ok(Self::SocketClose),
            8 => Ok(Self::ReflectOpen),
            9 => Ok(Self::ReflectClose),
            _ => Err(WinDivertValueError::Event(value)),
        }
    }
}

impl From<WinDivertEvent> for u8 {
    fn from(value: WinDivertEvent) -> Self {
        match value {
            WinDivertEvent::NetworkPacket => 0,
            WinDivertEvent::FlowStablished => 1,
            WinDivertEvent::FlowDeleted => 2,
            WinDivertEvent::SocketBind => 3,
            WinDivertEvent::SocketConnect => 4,
            WinDivertEvent::SocketListen => 5,
            WinDivertEvent::SocketAccept => 6,
            WinDivertEvent::SocketClose => 7,
            WinDivertEvent::ReflectOpen => 8,
            WinDivertEvent::ReflectClose => 9,
        }
    }
}

impl From<WinDivertEvent> for u32 {
    fn from(value: WinDivertEvent) -> Self {
        u8::from(value) as u32
    }
}

/**
WinDivert shutdown mode.
*/
#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum WinDivertShutdownMode {
    /// Stops new packets being queued for [`WinDivertRecv`](fn@super::WinDivertRecv)
    Recv = 1,
    /// Stops new packets being injected via [`WinDivertSend`](fn@super::WinDivertSend)
    Send = 2,
    /// Equivalent to [`WinDivertShutdownMode::Recv`] | [`WinDivertShutdownMode::Send`]
    Both = 3,
}

impl TryFrom<u32> for WinDivertShutdownMode {
    type Error = WinDivertValueError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(WinDivertShutdownMode::Recv),
            2 => Ok(WinDivertShutdownMode::Send),
            3 => Ok(WinDivertShutdownMode::Both),
            _ => Err(WinDivertValueError::Shutdown(value)),
        }
    }
}

impl From<WinDivertShutdownMode> for u32 {
    fn from(value: WinDivertShutdownMode) -> Self {
        match value {
            WinDivertShutdownMode::Recv => 1,
            WinDivertShutdownMode::Send => 2,
            WinDivertShutdownMode::Both => 3,
        }
    }
}

/**
WinDivert parameter enum.

Used to specify the parameter in [`WinDivertGetParam()`](fn@super::WinDivertGetParam) and [`WinDivertSetParam()`](fn@super::WinDivertSetParam).
*/
#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum WinDivertParam {
    /**
    WINDIVERT_PARAM_QUEUE_TIME parameter.

    Sets the maximum length of the packet queue for [`WinDivertRecv()`](fn@super::WinDivertRecv).

    The range of valid values goes from [`WINDIVERT_PARAM_QUEUE_LENGTH_MIN`](value@super::WINDIVERT_PARAM_QUEUE_LENGTH_MIN) to [`WINDIVERT_PARAM_QUEUE_LENGTH_MAX`](value@super::WINDIVERT_PARAM_QUEUE_LENGTH_MAX), with a default value of [`WINDIVERT_PARAM_QUEUE_LENGTH_DEFAULT`](`value@super::WINDIVERT_PARAM_QUEUE_LENGTH_DEFAULT`).
    */
    QueueLength = 0,
    /**
    WINDIVERT_PARAM_QUEUE_LENGTH parameter.

    Sets the minimum time, in milliseconds, a packet can be queued before it is automatically dropped. Packets cannot be queued indefinitely, and ideally, packets should be processed by the application as soon as is possible. Note that this sets the minimum time a packet can be queued before it can be dropped. The actual time may be exceed this value.

    The range of valid values goes from [`WINDIVERT_PARAM_QUEUE_TIME_MIN`](value@super::WINDIVERT_PARAM_QUEUE_TIME_MIN) to [`WINDIVERT_PARAM_QUEUE_TIME_MAX`](value@super::WINDIVERT_PARAM_QUEUE_TIME_MAX), with a fefault value of [`WINDIVERT_PARAM_QUEUE_TIME_DEFAULT`](`value@super::WINDIVERT_PARAM_QUEUE_TIME_DEFAULT`).
    */
    QueueTime = 1,
    /**
    WINDIVERT_PARAM_QUEUE_SIZE parameter.

    Sets the maximum number of bytes that can be stored in the packet queue for [`WinDivertRecv()`](fn@super::WinDivertRecv).

    The range of valid values goes from [`WINDIVERT_PARAM_QUEUE_SIZE_MIN`](value@super::WINDIVERT_PARAM_QUEUE_SIZE_MIN) to [`WINDIVERT_PARAM_QUEUE_SIZE_MAX`](value@super::WINDIVERT_PARAM_QUEUE_SIZE_MAX), with a fefault value of [`WINDIVERT_PARAM_QUEUE_SIZE_DEFAULT`](`value@super::WINDIVERT_PARAM_QUEUE_SIZE_DEFAULT`).
    */
    QueueSize = 2,
    /// Obtains the major version of the driver.
    VersionMajor = 3,
    /// Obtains the minor version of the driver.
    VersionMinor = 4,
}

impl TryFrom<u32> for WinDivertParam {
    type Error = WinDivertValueError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(WinDivertParam::QueueLength),
            1 => Ok(WinDivertParam::QueueTime),
            2 => Ok(WinDivertParam::QueueSize),
            3 => Ok(WinDivertParam::VersionMajor),
            4 => Ok(WinDivertParam::VersionMinor),
            _ => Err(WinDivertValueError::Parameter(value)),
        }
    }
}

impl From<WinDivertParam> for u32 {
    fn from(value: WinDivertParam) -> Self {
        match value {
            WinDivertParam::QueueLength => 0,
            WinDivertParam::QueueTime => 1,
            WinDivertParam::QueueSize => 2,
            WinDivertParam::VersionMajor => 3,
            WinDivertParam::VersionMinor => 4,
        }
    }
}

/**
Flag type required by [`WinDivertOpen()`](fn@super::WinDivertOpen). It follows a builder like style.

Different flags affect how the opened handle behaves. The following flags are supported:
 * `sniff`: This flag opens the WinDivert handle in `packet sniffing` mode. In packet sniffing mode the original packet is not dropped-and-diverted (the default) but copied-and-diverted. This mode is useful for implementing packet sniffing tools similar to those applications that currently use Winpcap.
 * `drop`: This flag indicates that the user application does not intend to read matching packets with [`recv()`](fn@super::WinDivertRecv) (or any of it's variants), instead the packets should be silently dropped. This is useful for implementing simple packet filters using the WinDivert [filter language](https://reqrypt.org/windivert-doc.html#filter_language).
 * `recv_only`: This flags forces the handle into receive only mode which effectively disables [`send()`](fn@super::WinDivertSend) (and any of it's variants). This means that it is possible to block/capture packets or events but not inject them.
 * `send_only`: This flags forces the handle into send only mode which effectively disables [`recv()`](fn@super::WinDivertRecv) (and any of it's variants). This means that it is possible to inject packets or events, but not block/capture them.
 * `no_installs`: This flags causes [`WinDivertOpen`](fn@super::WinDivertOpen) to fail with ERROR_SERVICE_DOES_NOT_EXIST (1060) if the WinDivert driver is not already installed. This flag is useful for querying the WinDivert driver state using [`Reflect`](super::WinDivertLayer::Reflect) layer.
 * `fragments`: If set, the handle will capture inbound IP fragments, but not inbound reassembled IP packets. Otherwise, if not set (the default), the handle will capture inbound reassembled IP packets, but not inbound IP fragments. This flag only affects inbound packets at the [`Network`](super::WinDivertLayer::Network) layer, else the flag is ignored.
Note that any combination of (`snif` | `drop`) or (`recv_only` | `send_only`) are considered invalid.

Some layers have mandatory flags:
 * [`WinDivertLayer::Flow`](type@WinDivertLayer::Flow): (`sniff` | `recv_only`)
 * [`WinDivertLayer::Socket`](type@WinDivertLayer::Socket): `recv_only`
 * [`WinDivertLayer::Reflect`](type@WinDivertLayer::Reflect): (`sniff` | `recv_only`)
*/
#[derive(Debug, Default, Copy, Clone)]
#[repr(transparent)]
pub struct WinDivertFlags(u64);

/// WinDivertFlags builder methods.
impl WinDivertFlags {
    pub(crate) const SNIFF: u64 = 0x0001;
    pub(crate) const DROP: u64 = 0x0002;
    pub(crate) const RECV_ONLY: u64 = 0x0004;
    pub(crate) const SEND_ONLY: u64 = 0x0008;
    pub(crate) const NO_INSTALLS: u64 = 0x0010;
    pub(crate) const FRAGMENTS: u64 = 0x0020;

    /// Creates a new flag field with all options unset.
    #[inline]
    pub const fn new() -> Self {
        Self(0)
    }

    /// Create a flag field with default values for [`WinDivertLayer::Flow`](type@WinDivertLayer::Flow).
    #[inline]
    pub const fn flow_default() -> Self {
        Self(Self::SNIFF | Self::RECV_ONLY)
    }

    /// Create a flag field with default values for [`WinDivertLayer::Socket`](type@WinDivertLayer::Socket).
    #[inline]
    pub const fn socket_default() -> Self {
        Self(Self::RECV_ONLY)
    }

    /// Create a flag field with default values for [`WinDivertLayer::Reflect`](type@WinDivertLayer::Reflect).
    #[inline]
    pub const fn reflect_default() -> Self {
        Self(Self::SNIFF | Self::RECV_ONLY)
    }

    /// Sets `sniff` flag.
    #[inline]
    pub const fn set_sniff(mut self) -> Self {
        self.0 |= Self::SNIFF;
        self
    }

    /// Unsets `sniff` flag.
    #[inline]
    pub const fn unset_sniff(mut self) -> Self {
        self.0 &= !Self::SNIFF;
        self
    }

    /// Sets `sniff` flag to `value`.
    #[inline]
    pub fn set_sniff_value(&mut self, value: bool) {
        self.0 = (self.0 & !Self::SNIFF) | (value as u64);
    }

    /// Sets `drop` flag.
    #[inline]
    pub const fn set_drop(mut self) -> Self {
        self.0 |= Self::DROP;
        self
    }

    /// Unsets `drop` flag.
    #[inline]
    pub const fn unset_drop(mut self) -> Self {
        self.0 &= !Self::DROP;
        self
    }

    /// Sets `drop` flag to `value`.
    #[inline]
    pub fn set_drop_value(&mut self, value: bool) {
        self.0 = (self.0 & !Self::DROP) | ((value as u64) << 1);
    }

    /// Sets `recv_only` flag
    #[inline]
    pub const fn set_recv_only(mut self) -> Self {
        self.0 |= Self::RECV_ONLY;
        self
    }

    /// Unsets `recv_only` flag
    #[inline]
    pub const fn unset_recv_only(mut self) -> Self {
        self.0 &= !Self::RECV_ONLY;
        self
    }

    /// Sets `recv_only` flag to `value`.
    #[inline]
    pub fn set_recv_only_value(&mut self, value: bool) {
        self.0 = (self.0 & !Self::RECV_ONLY) | ((value as u64) << 2);
    }

    /// Sets `send_only` flag.
    #[inline]
    pub const fn set_send_only(mut self) -> Self {
        self.0 |= Self::SEND_ONLY;
        self
    }

    /// Unsets `send_only` flag.
    #[inline]
    pub const fn unset_send_only(mut self) -> Self {
        self.0 &= !Self::SEND_ONLY;
        self
    }

    /// Sets `send_only` flag to `value`.
    #[inline]
    pub fn set_send_only_value(&mut self, value: bool) {
        self.0 = (self.0 & !Self::SEND_ONLY) | ((value as u64) << 3);
    }

    /// Sets `no_installs` flag.
    #[inline]
    pub const fn set_no_installs(mut self) -> Self {
        self.0 |= Self::NO_INSTALLS;
        self
    }

    /// Unsets `no_installs` flag.
    #[inline]
    pub const fn unset_no_installs(mut self) -> Self {
        self.0 &= !Self::NO_INSTALLS;
        self
    }

    /// Sets `no_installs` flag to `value`.
    #[inline]
    pub fn set_no_installs_value(&mut self, value: bool) {
        self.0 = (self.0 & !Self::NO_INSTALLS) | ((value as u64) << 4);
    }

    /// Sets `fragments` flag.
    #[inline]
    pub const fn set_fragments(mut self) -> Self {
        self.0 |= Self::FRAGMENTS;
        self
    }

    /// Unsets `fragments` flag.
    #[inline]
    pub const fn unset_fragments(mut self) -> Self {
        self.0 &= !Self::FRAGMENTS;
        self
    }

    /// Sets `fragments` flag to `value`.
    #[inline]
    pub fn set_fragments_value(&mut self, value: bool) {
        self.0 = (self.0 & !Self::FRAGMENTS) | ((value as u64) << 5);
    }
}

impl From<WinDivertFlags> for u64 {
    fn from(flags: WinDivertFlags) -> Self {
        flags.0
    }
}

/**
Wrapper helper struct around u64.

The type uses transparent representation to enforce using the provided methods to set the values of the flags used by [`WinDivertHelperCalcChecksums()`](fn@super::WinDivertHelperCalcChecksums)

The different flag values are:
 * `no_ip`: Do not calculate the IPv4 checksum.
 * `no_icmp`: Do not calculate the ICMP checksum.
 * `no_icmpv6`: Do not calculate the ICMPv6 checksum.
 * `no_tcp`: Do not calculate the TCP checksum.
 * `no_udp`: Do not calculate the UDP checksum.
*/
#[derive(Debug, Default, Copy, Clone)]
#[repr(transparent)]
pub struct ChecksumFlags(u64);

impl ChecksumFlags {
    pub(crate) const NO_IP: u64 = 0x0001;
    pub(crate) const NO_ICMP: u64 = 0x0002;
    pub(crate) const NO_ICMPV6: u64 = 0x0004;
    pub(crate) const NO_TCP: u64 = 0x0008;
    pub(crate) const NO_UDP: u64 = 0x0010;

    /// Creates a new flag field with default zero value.
    #[inline]
    pub const fn new() -> Self {
        Self(0)
    }

    /// Sets `no_ip` flag
    #[inline]
    pub const fn set_no_ip(mut self) -> Self {
        self.0 |= Self::NO_IP;
        self
    }

    /// Unsets `no_ip` flag
    #[inline]
    pub const fn unset_no_ip(mut self) -> Self {
        self.0 &= !Self::NO_IP;
        self
    }

    /// Sets `no_ip` flag to `value`.
    #[inline]
    pub fn set_no_ip_value(&mut self, value: bool) {
        self.0 = (self.0 & !Self::NO_IP) | (value as u64);
    }

    /// Sets `no_icmp` flag
    #[inline]
    pub const fn set_no_icmp(mut self) -> Self {
        self.0 |= Self::NO_ICMP;
        self
    }

    /// Unsets `no_icmp` flag
    #[inline]
    pub const fn unset_no_icmp(mut self) -> Self {
        self.0 &= !Self::NO_ICMP;
        self
    }

    /// Sets `no_icmp` flag to `value`.
    #[inline]
    pub fn set_no_icmp_value(&mut self, value: bool) {
        self.0 = (self.0 & !Self::NO_ICMP) | ((value as u64) << 1);
    }

    /// Sets `no_icmpv6` flag
    #[inline]
    pub const fn set_no_icmpv6(mut self) -> Self {
        self.0 |= Self::NO_ICMPV6;
        self
    }

    /// Unsets `no_icmpv6` flag
    #[inline]
    pub const fn unset_no_icmpv6(mut self) -> Self {
        self.0 &= !Self::NO_ICMPV6;
        self
    }

    /// Sets `no_icmpv6` flag to `value`.
    #[inline]
    pub fn set_no_icmpv6_value(&mut self, value: bool) {
        self.0 = (self.0 & !Self::NO_ICMPV6) | ((value as u64) << 2);
    }

    /// Sets `no_tcp` flag
    #[inline]
    pub const fn set_no_tcp(mut self) -> Self {
        self.0 |= Self::NO_TCP;
        self
    }

    /// Unsets `no_tcp` flag
    #[inline]
    pub const fn unset_no_tcp(mut self) -> Self {
        self.0 &= !Self::NO_TCP;
        self
    }

    /// Sets `no_tcp` flag to `value`.
    #[inline]
    pub fn set_no_tcp_value(&mut self, value: bool) {
        self.0 = (self.0 & !Self::NO_TCP) | ((value as u64) << 3);
    }

    /// Sets `no_udp` flag
    #[inline]
    pub const fn set_no_udp(mut self) -> Self {
        self.0 |= Self::NO_UDP;
        self
    }

    /// Unsets `no_udp` flag
    #[inline]
    pub const fn unset_no_udp(mut self) -> Self {
        self.0 &= !Self::NO_UDP;
        self
    }

    /// Sets `no_udp` flag to `value`.
    #[inline]
    pub fn set_no_udp_value(&mut self, value: bool) {
        self.0 = (self.0 & !Self::NO_UDP) | ((value as u64) << 4);
    }
}

impl From<ChecksumFlags> for u64 {
    fn from(flags: ChecksumFlags) -> Self {
        flags.0
    }
}

#[cfg(test)]
mod test {
    mod divert_flags {
        use super::super::*;

        macro_rules! test_flag {
            ($flag:ident, $function_set:ident, $function_unset:ident, $function_set_value:ident) => {
                #[test]
                fn $function_set() {
                    let flags = WinDivertFlags(
                        (WinDivertFlags::SNIFF
                            | WinDivertFlags::DROP
                            | WinDivertFlags::RECV_ONLY
                            | WinDivertFlags::SEND_ONLY
                            | WinDivertFlags::NO_INSTALLS
                            | WinDivertFlags::FRAGMENTS)
                            ^ WinDivertFlags::$flag,
                    );
                    assert_eq!(0, flags.0 & WinDivertFlags::$flag);
                    let new_flags = flags.$function_set();
                    assert_eq!(WinDivertFlags::$flag, new_flags.0 & WinDivertFlags::$flag);
                    assert_eq!(flags.0, new_flags.0 & !WinDivertFlags::$flag);
                }

                #[test]
                fn $function_unset() {
                    let flags = WinDivertFlags(
                        WinDivertFlags::SNIFF
                            | WinDivertFlags::DROP
                            | WinDivertFlags::RECV_ONLY
                            | WinDivertFlags::SEND_ONLY
                            | WinDivertFlags::NO_INSTALLS
                            | WinDivertFlags::FRAGMENTS,
                    );
                    assert_eq!(WinDivertFlags::$flag, flags.0 & WinDivertFlags::$flag);
                    let new_flags = flags.$function_unset();
                    assert_eq!(0, new_flags.0 & WinDivertFlags::$flag);
                    assert_eq!(flags.0, new_flags.0 | WinDivertFlags::$flag);
                }

                #[test]
                fn $function_set_value() {
                    let flags = WinDivertFlags(
                        WinDivertFlags::SNIFF
                            | WinDivertFlags::DROP
                            | WinDivertFlags::RECV_ONLY
                            | WinDivertFlags::SEND_ONLY
                            | WinDivertFlags::NO_INSTALLS
                            | WinDivertFlags::FRAGMENTS,
                    );
                    let mut new_flags = flags;
                    assert_eq!(WinDivertFlags::$flag, flags.0 & WinDivertFlags::$flag);
                    new_flags.$function_set_value(false);
                    assert_eq!(0, new_flags.0 & WinDivertFlags::$flag);
                    assert_eq!(flags.0, new_flags.0 | WinDivertFlags::$flag);
                    let flags = new_flags;
                    new_flags.$function_set_value(true);
                    assert_eq!(WinDivertFlags::$flag, new_flags.0 & WinDivertFlags::$flag);
                    assert_eq!(flags.0, new_flags.0 & !WinDivertFlags::$flag);
                }
            };
        }

        test_flag!(SNIFF, set_sniff, unset_sniff, set_sniff_value);
        test_flag!(DROP, set_drop, unset_drop, set_drop_value);
        test_flag!(
            RECV_ONLY,
            set_recv_only,
            unset_recv_only,
            set_recv_only_value
        );
        test_flag!(
            SEND_ONLY,
            set_send_only,
            unset_send_only,
            set_send_only_value
        );
        test_flag!(
            NO_INSTALLS,
            set_no_installs,
            unset_no_installs,
            set_no_installs_value
        );
        test_flag!(
            FRAGMENTS,
            set_fragments,
            unset_fragments,
            set_fragments_value
        );
    }

    mod checksum {
        use super::super::*;

        macro_rules! test_flag {
            ($flag:ident, $function_set:ident, $function_unset:ident, $function_set_value:ident) => {
                #[test]
                fn $function_set() {
                    let flags = ChecksumFlags(
                        (ChecksumFlags::NO_IP
                            | ChecksumFlags::NO_ICMP
                            | ChecksumFlags::NO_ICMPV6
                            | ChecksumFlags::NO_TCP
                            | ChecksumFlags::NO_UDP)
                            ^ ChecksumFlags::$flag,
                    );
                    assert_eq!(0, flags.0 & ChecksumFlags::$flag);
                    let new_flags = flags.$function_set();
                    assert_eq!(ChecksumFlags::$flag, new_flags.0 & ChecksumFlags::$flag);
                    assert_eq!(flags.0, new_flags.0 & !ChecksumFlags::$flag);
                }

                #[test]
                fn $function_unset() {
                    let flags = ChecksumFlags(
                        ChecksumFlags::NO_IP
                            | ChecksumFlags::NO_ICMP
                            | ChecksumFlags::NO_ICMPV6
                            | ChecksumFlags::NO_TCP
                            | ChecksumFlags::NO_UDP,
                    );
                    assert_eq!(ChecksumFlags::$flag, flags.0 & ChecksumFlags::$flag);
                    let new_flags = flags.$function_unset();
                    assert_eq!(0, new_flags.0 & ChecksumFlags::$flag);
                    assert_eq!(flags.0, new_flags.0 | ChecksumFlags::$flag);
                }

                #[test]
                fn $function_set_value() {
                    let flags = ChecksumFlags(
                        ChecksumFlags::NO_IP
                            | ChecksumFlags::NO_ICMP
                            | ChecksumFlags::NO_ICMPV6
                            | ChecksumFlags::NO_TCP
                            | ChecksumFlags::NO_UDP,
                    );
                    let mut new_flags = flags;
                    assert_eq!(ChecksumFlags::$flag, flags.0 & ChecksumFlags::$flag);
                    new_flags.$function_set_value(false);
                    assert_eq!(0, new_flags.0 & ChecksumFlags::$flag);
                    assert_eq!(flags.0, new_flags.0 | ChecksumFlags::$flag);
                    let flags = new_flags;
                    new_flags.$function_set_value(true);
                    assert_eq!(ChecksumFlags::$flag, new_flags.0 & ChecksumFlags::$flag);
                    assert_eq!(flags.0, new_flags.0 & !ChecksumFlags::$flag);
                }
            };
        }

        test_flag!(NO_IP, set_no_ip, unset_no_ip, set_no_ip_value);
        test_flag!(NO_ICMP, set_no_icmp, unset_no_icmp, set_no_icmp_value);
        test_flag!(
            NO_ICMPV6,
            set_no_icmpv6,
            unset_no_icmpv6,
            set_no_icmpv6_value
        );
        test_flag!(NO_TCP, set_no_tcp, unset_no_tcp, set_no_tcp_value);
        test_flag!(NO_UDP, set_no_udp, unset_no_udp, set_no_udp_value);
    }
}
