use std::convert::TryFrom;

use super::WinDivertError;

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
#[repr(C)]
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
    type Error = WinDivertError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(WinDivertLayer::Network),
            1 => Ok(WinDivertLayer::Forward),
            2 => Ok(WinDivertLayer::Flow),
            3 => Ok(WinDivertLayer::Socket),
            4 => Ok(WinDivertLayer::Reflect),
            _ => Err(WinDivertError::LayerValue),
        }
    }
}

impl From<WinDivertLayer> for u32 {
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

/**
WinDivert shutdown mode.
*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum WinDivertShutdownMode {
    None = 0,
    Recv = 1,
    Send = 2,
    Both = 3,
}

impl TryFrom<u32> for WinDivertShutdownMode {
    type Error = WinDivertError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(WinDivertShutdownMode::None),
            1 => Ok(WinDivertShutdownMode::Recv),
            2 => Ok(WinDivertShutdownMode::Send),
            3 => Ok(WinDivertShutdownMode::Both),
            _ => Err(WinDivertError::ShutdownValue),
        }
    }
}

impl From<WinDivertShutdownMode> for u32 {
    fn from(value: WinDivertShutdownMode) -> Self {
        match value {
            WinDivertShutdownMode::None => 0,
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
    VersionMajor = 3,
    VersionMinor = 4,
}

impl TryFrom<u32> for WinDivertParam {
    type Error = WinDivertError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(WinDivertParam::QueueLength),
            1 => Ok(WinDivertParam::QueueTime),
            2 => Ok(WinDivertParam::QueueSize),
            3 => Ok(WinDivertParam::VersionMajor),
            4 => Ok(WinDivertParam::VersionMinor),
            _ => Err(WinDivertError::ParameterValue),
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_no_ip(mut self) -> Self {
        self.0 |= 0x0001;
        self
    }

    pub fn unset_no_ip(mut self) -> Self {
        self.0 &= 0xFFFE;
        self
    }

    pub fn set_no_icmp(mut self) -> Self {
        self.0 &= 0x0002;
        self
    }

    pub fn unset_no_icmp(mut self) -> Self {
        self.0 ^= 0xFFFD;
        self
    }

    pub fn set_no_icmpv6(mut self) -> Self {
        self.0 &= 0x0004;
        self
    }

    pub fn unset_no_icmpv6(mut self) -> Self {
        self.0 ^= 0xFFFB;
        self
    }

    pub fn set_tcp(mut self) -> Self {
        self.0 &= 0x0008;
        self
    }

    pub fn unset_tcp(mut self) -> Self {
        self.0 ^= 0xFFF7;
        self
    }

    pub fn set_udp(mut self) -> Self {
        self.0 &= 0x0010;
        self
    }

    pub fn unset_udp(mut self) -> Self {
        self.0 ^= 0xFFEF;
        self
    }
}

impl From<ChecksumFlags> for u64 {
    fn from(flags: ChecksumFlags) -> Self {
        flags.0
    }
}
