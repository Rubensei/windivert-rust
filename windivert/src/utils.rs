use std::borrow::Cow;

pub(crate) fn prepare_internet_slice_data(slice: &[u8]) -> (&[u8], Cow<[u8]>) {
    let headers = etherparse::SlicedPacket::from_ip(slice)
        .expect("WinDivert can't capture anything below ip");
    let offset = match headers.net.unwrap() {
        etherparse::NetSlice::Ipv4(ip4slice) => ip4slice.header().total_len() as usize,
        etherparse::NetSlice::Ipv6(ip6slice) => ip6slice.header().payload_length() as usize + 40,
    };
    let (data, tail) = slice.split_at(offset);
    (tail, Cow::Borrowed(data))
}
