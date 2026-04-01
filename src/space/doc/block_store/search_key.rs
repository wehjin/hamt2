use crate::space::TableAddr;
use bytes::Bytes;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SearchKey {
    key_u32: u32,
}

impl SearchKey {
    pub fn from_addr(addr: &TableAddr) -> Self {
        Self {
            key_u32: addr.to_u32() & 0xFFFFFF00,
        }
    }
    pub fn next(mut self) -> Option<Self> {
        if self.key_u32 == 0 {
            None
        } else {
            self.key_u32 -= 0x100;
            Some(self)
        }
    }
}

impl Into<Bytes> for SearchKey {
    fn into(self) -> Bytes {
        let be_bytes = self.key_u32.to_be_bytes();
        (&be_bytes[..3]).to_vec().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn search_key_into_bytes() {
        let key = SearchKey::from_addr(&TableAddr::from(0x01020304u32));
        let bytes: Bytes = key.into();
        assert_eq!(&[0x01, 0x02, 0x03], bytes.as_ref());
    }

    #[test]
    fn search_key_next() {
        let mut key = SearchKey::from_addr(&TableAddr::from(0x00000104u32));
        key = key.next().expect("Failed to get next key");
        {
            let key_bytes: Bytes = key.clone().into();
            assert_eq!(&[0u8; 3], key_bytes.as_ref());
        }
        let final_key = key.next();
        assert_eq!(None, final_key);
    }
}
