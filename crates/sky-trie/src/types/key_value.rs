use crate::{HashKey, MapBase, TrieKey, TrieValue};
use std::sync::Arc;

const INT_KEY: u32 = 0x0000_0000;
const TRIE_KEY: u32 = 0x4000_0000;
const BYTES_KEY: u32 = 0x8000_0000;
const KEY_MASK: u32 = TrieKey::MASK;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum KeyValue {
    Int { key: u32, value: u32 },
    Subtrie { key: u32, value: MapBase },
    Bytes { key: u32, value: Arc<[u8]> },
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
    pub async fn from_hash_key_trie_value(key: HashKey, value: TrieValue) -> KeyValue {
        Self::from_trie_key_trie_value(key.i32(), value).await
    }
    pub async fn from_trie_key_trie_value(key: i32, value: TrieValue) -> KeyValue {
        let key = key as u32;
        debug_assert_eq!(key & KEY_MASK, key, "keys are 30 bits");
        let value = match value {
            TrieValue::U32(n) => Self::Int {
                key: key | INT_KEY,
                value: n,
            },
            TrieValue::SubTrie(map_base) => Self::Subtrie {
                key: key | TRIE_KEY,
                value: map_base,
            },
            TrieValue::Bytes(bytes) => Self::Bytes {
                key: key | BYTES_KEY,
                value: bytes.into(),
            },
        };
        value
    }
    pub fn to_trie_key_trie_value(&self) -> (i32, TrieValue) {
        let key = self.to_i32_key();
        match self {
            KeyValue::Int { value, .. } => {
                let value = TrieValue::U32(*value);
                (key, value)
            }
            KeyValue::Subtrie { value, .. } => {
                let map_base = value.clone();
                let value = TrieValue::SubTrie(map_base);
                (key, value)
            }
            KeyValue::Bytes { value, .. } => {
                let bytes = value.to_vec();
                let value = TrieValue::Bytes(bytes);
                (key, value)
            }
        }
    }
    pub async fn replace_value(self, value: TrieValue) -> Self {
        let key = self.to_i32_key();
        Self::from_trie_key_trie_value(key, value).await
    }
}
