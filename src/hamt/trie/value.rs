use crate::client::TransactError;
use crate::hamt::trie::mem::core::MemMapBase;
use std::fmt;
use std::fmt::Formatter;

#[derive(Clone, PartialEq, Eq)]
pub enum TrieValue {
    U32(u32),
    Subtrie(MemMapBase),
}

impl fmt::Debug for TrieValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TrieValue::U32(v) => f
                .debug_tuple("TrieValue")
                .field(&format_args!("U32[{}]", *v >> 1))
                .finish(),
            TrieValue::Subtrie(_) => f
                .debug_tuple("TrieValue")
                .field(&format_args!("Subtrie"))
                .finish(),
        }
    }
}

impl TrieValue {
    pub fn new(value: u32) -> Result<Self, TransactError> {
        if value & 0x80000000 != 0 {
            return Err(TransactError::HighBitInValue(value));
        }
        let value = value << 1;

        Ok(Self::U32(value))
    }
    pub fn to_u32(&self) -> u32 {
        let Self::U32(v) = self else {
            panic!("tried to convert non-u32 value to u32");
        };
        *v >> 1
    }
}
