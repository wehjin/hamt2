use crate::MapBase;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TrieValue {
    U32(u32),
    SubTrie(MapBase),
    Bytes(Vec<u8>),
}

impl From<u32> for TrieValue {
    fn from(v: u32) -> Self {
        Self::U32(v)
    }
}

impl From<&[u8]> for TrieValue {
    fn from(value: &[u8]) -> Self {
        let vec = Vec::from(value);
        TrieValue::Bytes(vec)
    }
}

impl<const N: usize> From<[u8; N]> for TrieValue {
    fn from(v: [u8; N]) -> Self {
        Self::Bytes(v.to_vec())
    }
}

impl From<&Vec<u8>> for TrieValue {
    fn from(value: &Vec<u8>) -> Self {
        TrieValue::Bytes(value.to_owned())
    }
}
impl From<Vec<u8>> for TrieValue {
    fn from(value: Vec<u8>) -> Self {
        TrieValue::Bytes(value)
    }
}
impl From<&str> for TrieValue {
    fn from(value: &str) -> Self {
        let vec = value.as_bytes().to_vec();
        TrieValue::Bytes(vec)
    }
}
impl From<&String> for TrieValue {
    fn from(value: &String) -> Self {
        let vec = value.as_bytes().to_vec();
        TrieValue::Bytes(vec)
    }
}
impl From<String> for TrieValue {
    fn from(value: String) -> Self {
        let vec = value.as_bytes().to_vec();
        TrieValue::Bytes(vec)
    }
}
