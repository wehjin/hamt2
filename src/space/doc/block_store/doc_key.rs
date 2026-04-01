use super::block_key::BlockKey;
use crate::space::doc::block_store::search_key::SearchKey;
use bytes::{Bytes, BytesMut};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DocKey {
    Details,
    Block(BlockKey),
    Search(SearchKey),
}

impl DocKey {
    const PREFIX_DETAILS: &'static [u8] = &[0x00];
    const PREFIX_BLOCK: &'static [u8] = &[0x01];
    pub fn into_block_key(self) -> BlockKey {
        match self {
            DocKey::Block(key) => key,
            _ => panic!("IrohKey is not a block key"),
        }
    }
    pub fn into_search_key(self) -> SearchKey {
        match self {
            DocKey::Search(key) => key,
            _ => panic!("IrohKey is not a search key"),
        }
    }
}

impl Into<Bytes> for DocKey {
    fn into(self) -> Bytes {
        match self {
            DocKey::Details => Bytes::from_static(Self::PREFIX_DETAILS),
            DocKey::Block(key) => {
                let mut bytes = BytesMut::with_capacity(1 + size_of::<BlockKey>());
                bytes.extend_from_slice(Self::PREFIX_BLOCK);
                {
                    let key_bytes: Bytes = key.into();
                    bytes.extend_from_slice(key_bytes.as_ref());
                }
                bytes.freeze()
            }
            DocKey::Search(key) => {
                let mut bytes = BytesMut::with_capacity(4);
                bytes.extend_from_slice(Self::PREFIX_BLOCK);
                {
                    let key_bytes: Bytes = key.into();
                    bytes.extend_from_slice(key_bytes.as_ref());
                }
                bytes.freeze()
            }
        }
    }
}

impl From<&[u8]> for DocKey {
    fn from(value: &[u8]) -> Self {
        debug_assert_eq!(Self::PREFIX_BLOCK, &value[..1]);
        let block_key = BlockKey::from(&value[1..]);
        DocKey::Block(block_key)
    }
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::space::TableAddr;
	#[test]
    fn into_bytes_for_block() {
        let addr = TableAddr::from(0x01020304u32);
        let iroh_key = DocKey::Block(BlockKey::new(addr, 0x05060708));
        let bytes: Bytes = iroh_key.into();
        assert_eq!(
            &[0x01, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
            bytes.as_ref()
        );
    }
    #[test]
    fn from_slice_for_block() {
        let bytes = Bytes::from(&[0x01, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08][..]);
        let iroh_key: DocKey = bytes.as_ref().into();
        assert_eq!(
            iroh_key,
            DocKey::Block(BlockKey::new(TableAddr::from(0x01020304u32), 0x05060708))
        );
    }

    #[test]
    fn into_bytes_for_details() {
        let iroh_key = DocKey::Details;
        let bytes: Bytes = iroh_key.into();
        assert_eq!(&[0x00], bytes.as_ref());
    }

    #[test]
    fn into_bytes_for_search() {
        let iroh_key = DocKey::Search(SearchKey::from_addr(&TableAddr::from(0x01020304u32)));
        let bytes: Bytes = iroh_key.into();
        assert_eq!(&[0x01, 0x01, 0x02, 0x03], bytes.as_ref());
    }
}
