use crate::space::TableAddr;
use bytes::{Bytes, BytesMut};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BlockKey {
    addr: TableAddr,
    len: u32,
}

impl BlockKey {
    pub fn new(addr: TableAddr, len: u32) -> Self {
        Self { addr, len }
    }
    pub fn handles_addr(&self, addr: TableAddr) -> bool {
        addr >= self.addr && addr < (self.addr + self.len as usize)
    }
    pub fn to_addr(&self) -> TableAddr {
        self.addr
    }
}

impl Into<Bytes> for BlockKey {
    fn into(self) -> Bytes {
        let mut buf = BytesMut::with_capacity(2 * std::mem::size_of::<u32>());
        let addr_u32 = self.addr.to_u32();
        buf.extend_from_slice(&addr_u32.to_be_bytes());
        buf.extend_from_slice(&self.len.to_be_bytes());
        buf.freeze()
    }
}
impl From<&[u8]> for BlockKey {
    fn from(value: &[u8]) -> Self {
        let addr_u32 = u32::from_be_bytes(
            (&value[..4])
                .try_into()
                .expect("Failed to convert addr bytes to u32"),
        );
        let len_u32 = u32::from_be_bytes(
            (&value[4..])
                .try_into()
                .expect("Failed to convert len bytes to u32"),
        );
        Self {
            addr: TableAddr::from(addr_u32),
            len: len_u32,
        }
    }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
    fn block_key_into_bytes() {
        let addr_u32 = 0x01020304u32;
        let block_key = BlockKey {
            addr: TableAddr::from(addr_u32),
            len: 0x05060708,
        };
        let bytes: Bytes = block_key.into();
        assert_eq!(
            vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
            bytes.to_vec()
        );
    }

    #[test]
    fn block_key_from_bytes() {
        let bytes = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let block_key: BlockKey = BlockKey::from(bytes.as_slice());
        assert_eq!(TableAddr::from(0x01020304u32), block_key.addr);
        assert_eq!(0x05060708, block_key.len);
    }
}
