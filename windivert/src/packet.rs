use std::fmt::Debug;

use windivert_sys::address::WINDIVERT_ADDRESS;

/// Temporary packet abstraction storing all the data windivert returns from recv for each "match".
#[derive(Debug, Clone)]
pub struct Packet {
    pub address: WINDIVERT_ADDRESS,
    pub data: Vec<u8>,
}
