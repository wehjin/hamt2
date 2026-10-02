use crate::trie::{
    Buffer, BufferIndex, BufferMut, HashKey, TrieInsertError, TrieKey, TrieValue,
    get_bytes_from_buffer, push_bytes_to_buffer,
};
use serde::{Deserialize, Serialize};

const INT_KEY: u32 = 0x0000_0000;
const TRIE_KEY: u32 = 0x4000_0000;
const BYTES_KEY: u32 = 0x8000_0000;
const KEY_MASK: u32 = TrieKey::MASK;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum KeyValue {
    Int { key: u32, value: u32 },
    Subtrie { key: u32, value: BufferIndex },
    Bytes { key: u32, value: BufferIndex },
}

impl KeyValue {
    pub fn to_i32_key(&self) -> i32 {
        let key = match self {
            KeyValue::Int { key, .. } => *key,
            KeyValue::Subtrie { key, .. } => *key,
            KeyValue::Bytes { key, .. } => *key,
        };
        (key & KEY_MASK) as i32
    }
    pub async fn from_hash_key_trie_value(
        key: HashKey,
        value: TrieValue,
        buffer: &mut impl BufferMut,
    ) -> Result<KeyValue, TrieInsertError> {
        Self::from_trie_key_trie_value(key.i32(), value, buffer).await
    }
    pub async fn from_trie_key_trie_value(
        key: i32,
        value: TrieValue,
        buffer: &mut impl BufferMut,
    ) -> Result<KeyValue, TrieInsertError> {
        let key = key as u32;
        debug_assert_eq!(key & KEY_MASK, key, "keys are 30 bits");
        let value = match value {
            TrieValue::U32(n) => Self::Int {
                key: key | INT_KEY,
                value: n,
            },
            TrieValue::SubTrie(map_base) => Self::Subtrie {
                key: key | TRIE_KEY,
                value: buffer.push_subtrie(map_base).await?,
            },
            TrieValue::Bytes(bytes) => Self::Bytes {
                key: key | BYTES_KEY,
                value: push_bytes_to_buffer(bytes.as_slice(), buffer).await?,
            },
        };
        Ok(value)
    }
    pub async fn to_trie_key_trie_value(&self, buffer: &impl Buffer) -> (i32, TrieValue) {
        let key = self.to_i32_key();
        match self {
            KeyValue::Int { value, .. } => {
                let value = TrieValue::U32(*value);
                (key, value)
            }
            KeyValue::Subtrie { value, .. } => {
                let map_base = buffer.get_subtrie(*value).await;
                let value = TrieValue::SubTrie(map_base);
                (key, value)
            }
            KeyValue::Bytes { value, .. } => {
                let bytes = get_bytes_from_buffer(*value, buffer).await;
                let value = TrieValue::Bytes(bytes);
                (key, value)
            }
        }
    }
    pub async fn replace_value(
        self,
        value: TrieValue,
        buffer: &mut impl BufferMut,
    ) -> Result<Self, TrieInsertError> {
        let key = self.to_i32_key();
        Self::from_trie_key_trie_value(key, value, buffer).await
    }
}
