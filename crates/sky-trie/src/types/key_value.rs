use crate::{HashKey, MapBase, SkyTrie, TrieValue};
use std::sync::Arc;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum KeyValue {
    Int { key: i32, value: u32 },
    Subtrie { key: i32, value: MapBase },
    Bytes { key: i32, value: Arc<[u8]> },
    Nil { key: i32 },
}

impl KeyValue {
    pub fn to_i32_key(&self) -> i32 {
        match self {
            KeyValue::Int { key, .. } => *key,
            KeyValue::Subtrie { key, .. } => *key,
            KeyValue::Bytes { key, .. } => *key,
            KeyValue::Nil { key } => *key,
        }
    }
    pub fn from_hash_key_trie_value(key: HashKey, value: TrieValue) -> KeyValue {
        Self::from_trie_key_trie_value(key.i32(), value)
    }
    pub fn from_trie_key_trie_value(key: i32, value: TrieValue) -> KeyValue {
        match value {
            TrieValue::U32(n) => Self::Int { key, value: n },
            TrieValue::Trie(sky_trie) => Self::Subtrie {
                key,
                value: sky_trie.root,
            },
            TrieValue::Bytes(bytes) => Self::Bytes {
                key,
                value: bytes.into(),
            },
            TrieValue::Nil => Self::Nil { key },
        }
    }
    pub fn to_trie_key_trie_value(&self) -> (i32, TrieValue) {
        let key = self.to_i32_key();
        match self {
            KeyValue::Int { value, .. } => {
                let value = TrieValue::U32(*value);
                (key, value)
            }
            KeyValue::Subtrie { value, .. } => {
                let value = TrieValue::Trie(SkyTrie { root: value.clone() });
                (key, value)
            }
            KeyValue::Bytes { value, .. } => {
                let bytes = value.to_vec();
                let value = TrieValue::Bytes(bytes);
                (key, value)
            }
            KeyValue::Nil { .. } => (key, TrieValue::Nil),
        }
    }
    pub fn replace_value(self, value: TrieValue) -> Self {
        let key = self.to_i32_key();
        Self::from_trie_key_trie_value(key, value)
    }
}
