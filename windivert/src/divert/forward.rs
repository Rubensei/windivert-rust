use std::borrow::Cow;

use etherparse::{InternetSlice, SlicedPacket};

use crate::address::WinDivertAddress;
use crate::prelude::*;

impl WinDivert<ForwardLayer> {
    /// WinDivert constructor for forward layer.
    pub fn forward(
        filter: impl AsRef<str>,
        priority: i16,
        flags: WinDivertFlags,
    ) -> Result<Self, WinDivertError> {
        Self::new(filter.as_ref(), WinDivertLayer::Forward, priority, flags)
    }

    /// Single packet blocking recv function.
    pub fn recv<'a>(
        &self,
        buffer: &'a mut [u8],
    ) -> Result<WinDivertPacket<'a, ForwardLayer>, WinDivertError> {
        self.internal_recv(Some(buffer))
    }

    /// Single packet blocking recv that won't error with [`WinDivertRecvError::InsufficientBuffer`] and will return a [partial packet](`WinDivertPartialPacket`) instead.
    pub fn partial_recv<'a>(
        &self,
        buffer: &'a mut [u8],
    ) -> Result<PacketEither<'a, ForwardLayer>, WinDivertError> {
        self.internal_partial_recv(Some(buffer))
    }

    /// Batched blocking recv function.
    pub fn recv_ex<'a>(
        &self,
        buffer: &'a mut [u8],
        packet_count: u8,
    ) -> Result<Vec<WinDivertPacket<'a, ForwardLayer>>, WinDivertError> {
        let (mut buffer, addresses) = self.internal_recv_ex(Some(buffer), packet_count)?;
        let mut packets = Vec::with_capacity(addresses.len());
        for addr in addresses.into_iter() {
            packets.push(WinDivertPacket {
                address: WinDivertAddress::<ForwardLayer>::from_raw(addr),
                data: buffer
                    .map(|inner_buffer| {
                        let headers = SlicedPacket::from_ip(inner_buffer)
                            .expect("WinDivert can't capture anything below ip");
                        let offset = match headers.ip.unwrap() {
                            InternetSlice::Ipv4(ip_header, _) => ip_header.total_len() as usize,
                            InternetSlice::Ipv6(ip6header, _) => {
                                ip6header.payload_length() as usize + 40
                            }
                        };
                        let (data, tail) = inner_buffer.split_at(offset);
                        buffer = Some(tail);
                        Cow::Borrowed(data)
                    })
                    .unwrap_or_default(),
            });
        }
        Ok(packets)
    }

    /// Single packet send function.
    pub fn send(&self, packet: &WinDivertPacket<ForwardLayer>) -> Result<u32, WinDivertError> {
        self.internal_send(packet)
    }

    /// Batched packet send function.
    /// Windivert only allows up to [`WINDIVERT_BATCH_MAX`](windivert_sys::WINDIVERT_BATCH_MAX) packets to be sent at once.
    pub fn send_ex<'packets, 'data: 'packets>(
        &self,
        packets: &'packets [WinDivertPacket<'data, ForwardLayer>],
    ) -> Result<u32, WinDivertError> {
        self.internal_send_ex(packets)
    }

    /// Single packet blocking recv function with timeout.
    pub fn recv_wait<'a>(
        &self,
        buffer: &'a mut [u8],
        timeout_ms: u32,
    ) -> Result<WinDivertPacket<'a, ForwardLayer>, WinDivertError> {
        if timeout_ms == 0 {
            self.internal_recv(Some(buffer))
        } else {
            self.internal_recv_wait_ex(Some(buffer), 1, timeout_ms)
                .map(|(data, addr)| WinDivertPacket {
                    address: WinDivertAddress::<ForwardLayer>::from_raw(addr[0]),
                    data: data.unwrap_or_default().into(),
                })
        }
    }

    /// Batched blocking recv function with timeout.
    pub fn recv_wait_ex<'a>(
        &self,
        buffer: &'a mut [u8],
        packet_count: u8,
        timeout_ms: u32,
    ) -> Result<Vec<WinDivertPacket<'a, ForwardLayer>>, WinDivertError> {
        let (mut buffer, addresses) = if timeout_ms == 0 {
            self.internal_recv_ex(Some(buffer), packet_count)?
        } else {
            self.internal_recv_wait_ex(Some(buffer), packet_count, timeout_ms)?
        };
        let mut packets = Vec::with_capacity(addresses.len());
        for addr in addresses.into_iter() {
            packets.push(WinDivertPacket {
                address: WinDivertAddress::<ForwardLayer>::from_raw(addr),
                data: buffer
                    .map(|inner_buffer| {
                        let headers = SlicedPacket::from_ip(inner_buffer)
                            .expect("WinDivert can't capture anything below ip");
                        let offset = match headers.ip.unwrap() {
                            InternetSlice::Ipv4(ip_header, _) => ip_header.total_len() as usize,
                            InternetSlice::Ipv6(ip6header, _) => {
                                ip6header.payload_length() as usize + 40
                            }
                        };
                        let (data, tail) = inner_buffer.split_at(offset);
                        buffer = Some(tail);
                        Cow::Borrowed(data)
                    })
                    .unwrap_or_default(),
            });
        }
        Ok(packets)
    }
}
