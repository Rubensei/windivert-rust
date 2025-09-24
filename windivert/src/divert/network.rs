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
    /// A timeout of 0 will return the queued data without blocking
    pub fn recv_wait<'a>(
        &self,
        buffer: &'a mut [u8],
        timeout_ms: u32,
    ) -> Result<WinDivertPacket<'a, NetworkLayer>, WinDivertError> {
        self.internal_recv_wait_ex(Some(buffer), 1, timeout_ms)
            .map(|(data, addr)| WinDivertPacket {
                address: WinDivertAddress::<NetworkLayer>::from_raw(addr[0]),
                data: data.unwrap_or_default().into(),
            })
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
    use serial_test::serial;
    use windows::Win32::Foundation::SetLastError;

    const EX_TEST_PACKET_COUNT: u8 = 5;

    fn setup_divert(sys_wrapper: SysWrapper) -> WinDivert<NetworkLayer> {
        WinDivert {
            handle: Arc::new(HANDLE(1usize as *mut c_void)),
            tls_index: TlsIndex::alloc_tls().unwrap(),
            core: sys_wrapper,
            _layer: PhantomData::<NetworkLayer>,
        }
    }

    #[test]
    fn recv_ok() {
        let mut sys_wrapper = SysWrapper::default();
        sys_wrapper.expect_WinDivertRecv().returning(
            |_, pPacket, packetLen, pRecvLen, address| unsafe {
                let mut buffer =
                    std::slice::from_raw_parts_mut(pPacket as *mut u8, packetLen as usize);
                *pRecvLen = buffer.write(crate::test_data::ECHO_REQUEST).unwrap() as u32;
                *address = WINDIVERT_ADDRESS::default();
                1
            },
        );
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
        sys_wrapper.expect_WinDivertRecv().returning(
            |_, pPacket, packetLen, pRecvLen, address| unsafe {
                let mut buffer =
                    std::slice::from_raw_parts_mut(pPacket as *mut u8, packetLen as usize);
                *pRecvLen = buffer.write(crate::test_data::ECHO_REQUEST).unwrap() as u32;
                *address = WINDIVERT_ADDRESS::default();
                1
            },
        );
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
        sys_wrapper.expect_WinDivertRecv().returning(
            |_, pPacket, packetLen, pRecvLen, address| unsafe {
                let mut buffer =
                    std::slice::from_raw_parts_mut(pPacket as *mut u8, packetLen as usize);
                *pRecvLen = buffer.write(crate::test_data::ECHO_REQUEST).unwrap() as u32;
                *address = WINDIVERT_ADDRESS::default();
                SetLastError(windows::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER);
                0
            },
        );
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

    #[test]
    fn recv_ex_ok() {
        let mut sys_wrapper = SysWrapper::default();
        sys_wrapper.expect_WinDivertRecvEx().returning(
            |_, pPacket, packetLen, pRecvLen, _, pAddr, pAddrLen, _| unsafe {
                let packet_count = *pAddrLen as usize / size_of::<WINDIVERT_ADDRESS>();
                assert!(packetLen as usize > packet_count * crate::test_data::ECHO_REQUEST.len());
                let buffer = std::slice::from_raw_parts_mut(pPacket as *mut u8, packetLen as usize);
                let addresses = std::slice::from_raw_parts_mut(pAddr, packet_count);
                let mut index = 0;
                for address in &mut addresses[..] {
                    *address = WINDIVERT_ADDRESS::default();

                    buffer[index..index + crate::test_data::ECHO_REQUEST.len()]
                        .copy_from_slice(crate::test_data::ECHO_REQUEST);
                    index += crate::test_data::ECHO_REQUEST.len();
                }
                *pRecvLen = index as u32;
                1
            },
        );
        let divert = setup_divert(sys_wrapper);
        let mut buffer = vec![0; 1500 * EX_TEST_PACKET_COUNT as usize];
        let packets = divert.recv_ex(&mut buffer[..], EX_TEST_PACKET_COUNT);
        assert!(packets.is_ok());
        let packets = packets.unwrap();
        assert_eq!(EX_TEST_PACKET_COUNT as usize, packets.len());
        for packet in packets.iter() {
            assert_eq!(packet.data[..], crate::test_data::ECHO_REQUEST[..]);
        }
    }

    #[test]
    #[serial]
    #[allow(non_snake_case)]
    fn recv_wait_ok() {
        let overlapped_ctx = Overlapped::init_context();
        overlapped_ctx.expect().returning(|_, _| {
            let mut overlapped = Overlapped::default();
            overlapped
                .expect_as_raw_mut()
                .returning(|| &mut 1u8 as *mut u8 as *mut c_void);
            overlapped
                .expect_wait_for_object()
                .returning(|_| Ok(Some(())));
            overlapped
                .expect_get_result()
                .returning(|| Ok(crate::test_data::ECHO_REQUEST.len() as u32));
            Ok(overlapped)
        });
        let mut sys_wrapper = SysWrapper::default();
        sys_wrapper.expect_WinDivertRecvEx().returning(
            |_, pPacket, packetLen, _, _, pAddr, pAddrLen, _| unsafe {
                assert_eq!(size_of::<WINDIVERT_ADDRESS>(), *pAddrLen as usize);
                assert!(packetLen as usize > crate::test_data::ECHO_REQUEST.len());
                let buffer = std::slice::from_raw_parts_mut(pPacket as *mut u8, packetLen as usize);
                *pAddr = WINDIVERT_ADDRESS::default();
                buffer[0..crate::test_data::ECHO_REQUEST.len()]
                    .copy_from_slice(&crate::test_data::ECHO_REQUEST[..]);
                SetLastError(ERROR_IO_PENDING);
                0
            },
        );
        let divert = setup_divert(sys_wrapper);
        let mut buffer = vec![0; 1500];
        let packet = divert.recv_wait(&mut buffer[..], 100);
        assert!(packet.is_ok());
        let packet = packet.unwrap();
        assert_eq!(packet.data[..], crate::test_data::ECHO_REQUEST[..]);
    }

    #[test]
    #[serial]
    #[allow(non_snake_case)]
    fn recv_ex_wait_ok() {
        let overlapped_ctx = Overlapped::init_context();
        overlapped_ctx.expect().returning(|_, _| {
            let mut overlapped = Overlapped::default();
            overlapped
                .expect_as_raw_mut()
                .returning(|| &mut 1u8 as *mut u8 as *mut c_void);
            overlapped
                .expect_wait_for_object()
                .returning(|_| Ok(Some(())));
            overlapped.expect_get_result().returning(|| {
                Ok(crate::test_data::ECHO_REQUEST.len() as u32 * EX_TEST_PACKET_COUNT as u32)
            });
            Ok(overlapped)
        });
        let mut sys_wrapper = SysWrapper::default();
        sys_wrapper.expect_WinDivertRecvEx().returning(
            |_, pPacket, packetLen, _, _, pAddr, pAddrLen, _| unsafe {
                let packet_count = *pAddrLen as usize / size_of::<WINDIVERT_ADDRESS>();
                assert!(packetLen as usize > packet_count * crate::test_data::ECHO_REQUEST.len());
                let buffer = std::slice::from_raw_parts_mut(pPacket as *mut u8, packetLen as usize);
                let addresses = std::slice::from_raw_parts_mut(pAddr, packet_count);
                let mut index = 0;
                for address in &mut addresses[..] {
                    *address = WINDIVERT_ADDRESS::default();

                    buffer[index..index + crate::test_data::ECHO_REQUEST.len()]
                        .copy_from_slice(crate::test_data::ECHO_REQUEST);
                    index += crate::test_data::ECHO_REQUEST.len();
                }
                SetLastError(ERROR_IO_PENDING);
                0
            },
        );
        let divert = setup_divert(sys_wrapper);
        let mut buffer = vec![0; 1500 * EX_TEST_PACKET_COUNT as usize];
        let packets = divert.recv_wait_ex(&mut buffer[..], EX_TEST_PACKET_COUNT, 100);
        assert!(packets.is_ok());
        let packets = packets.unwrap();
        assert_eq!(EX_TEST_PACKET_COUNT as usize, packets.len());
        for packet in packets.iter() {
            assert_eq!(packet.data[..], crate::test_data::ECHO_REQUEST[..]);
        }
    }

    #[test]
    fn send_ok() {
        let mut sys_wrapper = SysWrapper::default();
        sys_wrapper
            .expect_WinDivertSend()
            .returning(|_, pPacket, packetLen, pSendLen, _| unsafe {
                let buffer = std::slice::from_raw_parts(pPacket as *const u8, packetLen as usize);
                assert_eq!(buffer, crate::test_data::ECHO_REQUEST);
                *pSendLen = packetLen;
                1
            });
        let divert = setup_divert(sys_wrapper);
        let packet = WinDivertPacket {
            address: unsafe { WinDivertAddress::<NetworkLayer>::new() },
            data: Cow::Borrowed(crate::test_data::ECHO_REQUEST),
        };
        let result = divert.send(&packet);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), crate::test_data::ECHO_REQUEST.len() as u32);
    }

    #[test]
    fn send_ex_ok() {
        let mut sys_wrapper = SysWrapper::default();
        sys_wrapper.expect_WinDivertSendEx().returning(
            |_, pPacket, packetLen, pSendLen, _, _, _, _| unsafe {
                let buffer = std::slice::from_raw_parts(pPacket as *const u8, packetLen as usize);
                assert_eq!(
                    buffer,
                    crate::test_data::ECHO_REQUEST.repeat(EX_TEST_PACKET_COUNT as usize)
                );
                *pSendLen = packetLen;
                1
            },
        );
        let divert = setup_divert(sys_wrapper);
        let packets: Vec<_> = (0..EX_TEST_PACKET_COUNT)
            .map(|_| WinDivertPacket {
                address: unsafe { WinDivertAddress::<NetworkLayer>::new() },
                data: Cow::Borrowed(crate::test_data::ECHO_REQUEST),
            })
            .collect();
        let result = divert.send_ex(&packets);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            (crate::test_data::ECHO_REQUEST.len() * EX_TEST_PACKET_COUNT as usize) as u32
        );
    }
}
