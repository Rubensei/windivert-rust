use crate::address::WinDivertAddress;
use crate::utils::*;

use super::*;

impl WinDivert<NetworkLayer> {
    /// WinDivert constructor for network layer.
    pub fn network(
        filter: impl AsRef<str>,
        priority: i16,
        flags: WinDivertFlags,
    ) -> Result<Self, WinDivertError> {
        Self::new(filter.as_ref(), WinDivertLayer::Network, priority, flags)
    }

    /// Single packet blocking recv function.
    pub fn recv<'a>(
        &self,
        buffer: &'a mut [u8],
    ) -> Result<WinDivertPacket<'a, NetworkLayer>, WinDivertError> {
        self.internal_recv(Some(buffer))
    }

    /// Single packet blocking recv that won't error with [`WinDivertRecvError::InsufficientBuffer`] and will return a [partial packet](`WinDivertPartialPacket`) instead.
    pub fn partial_recv<'a>(
        &self,
        buffer: &'a mut [u8],
    ) -> Result<PacketEither<'a, NetworkLayer>, WinDivertError> {
        self.internal_partial_recv(Some(buffer))
    }

    /// Batched blocking recv function.
    pub fn recv_ex<'a>(
        &self,
        buffer: &'a mut [u8],
        packet_count: u8,
    ) -> Result<Vec<WinDivertPacket<'a, NetworkLayer>>, WinDivertError> {
        let (mut buffer, addresses) = self.internal_recv_ex(Some(buffer), packet_count)?;
        let mut packets = Vec::with_capacity(addresses.len());
        for addr in addresses.into_iter() {
            packets.push(WinDivertPacket {
                address: WinDivertAddress::<NetworkLayer>::from_raw(addr),
                data: buffer
                    .map(|inner_buffer| {
                        let (tail, data) = prepare_internet_slice_data(inner_buffer);
                        buffer = Some(tail);
                        data
                    })
                    .unwrap_or_default(),
            });
        }
        Ok(packets)
    }

    /// Single packet send function.
    pub fn send(&self, packet: &WinDivertPacket<NetworkLayer>) -> Result<u32, WinDivertError> {
        self.internal_send(packet)
    }

    /// Batched packet send function.
    /// Windivert only allows up to [`WINDIVERT_BATCH_MAX`](windivert_sys::WINDIVERT_BATCH_MAX) packets to be sent at once.
    pub fn send_ex<'packets, 'data: 'packets>(
        &self,
        packets: &'packets [WinDivertPacket<'data, NetworkLayer>],
    ) -> Result<u32, WinDivertError> {
        self.internal_send_ex(packets)
    }

    /// Single packet blocking recv function with timeout.
    pub fn recv_wait<'a>(
        &self,
        buffer: &'a mut [u8],
        timeout_ms: u32,
    ) -> Result<WinDivertPacket<'a, NetworkLayer>, WinDivertError> {
        if timeout_ms == 0 {
            self.internal_recv(Some(buffer))
        } else {
            self.internal_recv_wait_ex(Some(buffer), 1, timeout_ms)
                .map(|(data, addr)| WinDivertPacket {
                    address: WinDivertAddress::<NetworkLayer>::from_raw(addr[0]),
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
    ) -> Result<Vec<WinDivertPacket<'a, NetworkLayer>>, WinDivertError> {
        let (mut buffer, addresses) = if timeout_ms == 0 {
            self.internal_recv_ex(Some(buffer), packet_count)?
        } else {
            self.internal_recv_wait_ex(Some(buffer), packet_count, timeout_ms)?
        };
        let mut packets = Vec::with_capacity(addresses.len());
        for addr in addresses.into_iter() {
            packets.push(WinDivertPacket {
                address: WinDivertAddress::<NetworkLayer>::from_raw(addr),
                data: buffer
                    .map(|inner_buffer| {
                        let (tail, data) = prepare_internet_slice_data(inner_buffer);
                        buffer = Some(tail);
                        data
                    })
                    .unwrap_or_default(),
            });
        }
        Ok(packets)
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use std::io::Write;

    use super::*;
    use windows::Win32::Foundation::SetLastError;

    fn setup_divert(sys_wrapper: SysWrapper) -> WinDivert<NetworkLayer> {
        WinDivert {
            handle: HANDLE(1usize as *mut c_void),
            tls_index: TlsIndex::alloc_tls().unwrap(),
            core: sys_wrapper,
            _layer: PhantomData::<NetworkLayer>,
        }
    }

    #[test]
    fn recv_ok() {
        let mut sys_wrapper = SysWrapper::default();
        sys_wrapper
            .expect_WinDivertRecv()
            .returning(|_, pPacket, packetLen, pRecvLen, _| unsafe {
                let mut buffer =
                    std::slice::from_raw_parts_mut(pPacket as *mut u8, packetLen as usize);
                *pRecvLen = buffer.write(crate::test_data::ECHO_REQUEST).unwrap() as u32;
                1
            });
        let divert = setup_divert(sys_wrapper);
        let mut buffer = vec![0; 1500];
        let packet = divert.recv(&mut buffer[..]);
        assert!(packet.is_ok());
        let packet = packet.unwrap();
        assert_eq!(packet.data[..], crate::test_data::ECHO_REQUEST[..]);
    }

    #[test]
    fn partial_recv_full() {
        let mut sys_wrapper = SysWrapper::default();
        sys_wrapper
            .expect_WinDivertRecv()
            .returning(|_, pPacket, packetLen, pRecvLen, _| unsafe {
                let mut buffer =
                    std::slice::from_raw_parts_mut(pPacket as *mut u8, packetLen as usize);
                *pRecvLen = buffer.write(crate::test_data::ECHO_REQUEST).unwrap() as u32;
                1
            });
        let divert = setup_divert(sys_wrapper);
        let mut buffer = vec![0; 1500];
        let packet = divert.partial_recv(&mut buffer[..]);
        assert!(packet.is_ok());
        let packet = packet.unwrap();
        match packet {
            PacketEither::Partial(_) => assert!(false),
            PacketEither::Full(packet) => {
                assert_eq!(packet.data[..], crate::test_data::ECHO_REQUEST[..]);
            }
        };
    }

    #[test]
    fn partial_recv_partial() {
        let mut sys_wrapper = SysWrapper::default();
        sys_wrapper
            .expect_WinDivertRecv()
            .returning(|_, pPacket, packetLen, pRecvLen, _| unsafe {
                let mut buffer =
                    std::slice::from_raw_parts_mut(pPacket as *mut u8, packetLen as usize);
                *pRecvLen = buffer.write(crate::test_data::ECHO_REQUEST).unwrap() as u32;
                SetLastError(windows::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER);
                0
            });
        let divert = setup_divert(sys_wrapper);
        let mut buffer = vec![0; crate::test_data::ECHO_REQUEST.len() - 10];
        let packet = divert.partial_recv(&mut buffer[..]);
        assert!(packet.is_ok());
        let packet = packet.unwrap();
        match packet {
            PacketEither::Full(_) => assert!(false),
            PacketEither::Partial(partial_packet) => {
                assert_eq!(
                    partial_packet.data[..],
                    crate::test_data::ECHO_REQUEST[..partial_packet.data.len()]
                );
            }
        };
    }
}
