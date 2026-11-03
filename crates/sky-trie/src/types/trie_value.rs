use crate::SkyTrie;

/// The value stored at a trie key: a u32, a nested trie, raw bytes, or nil.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TrieValue {
    U32(u32),
    Trie(SkyTrie),
    Bytes(Vec<u8>),
    Nil,
}

impl TrieValue {
    /// Returns the value stored at `key`, or `Nil` if absent. Non-`Trie`
    /// variants are leaves with nothing below them.
    pub fn query(&self, key: i32) -> TrieValue {
        match self {
            TrieValue::Trie(trie) => trie.query(key).unwrap_or(TrieValue::Nil),
            _ => TrieValue::Nil,
        }
    }

    /// Returns the value stored at the deep path `key`, or `Nil` if absent.
    /// Non-`Trie` variants are leaves with nothing below them.
    pub fn query_deep<const N: usize>(&self, key: [i32; N]) -> TrieValue {
        match self {
            TrieValue::Trie(trie) => trie.query_deep(key).unwrap_or(TrieValue::Nil),
            _ => TrieValue::Nil,
        }
    }

    /// Returns all `(key, value)` pairs in this value's trie. `Nil` and an
    /// empty `Trie` yield an empty collection; leaf variants panic.
    pub fn query_all(&self) -> Vec<(i32, TrieValue)> {
        match self {
            TrieValue::Trie(trie) => trie.query_all(),
            TrieValue::Nil => Vec::new(),
            _ => panic!("cannot enumerate a leaf value"),
        }
    }

    /// Returns whether `key` is present. For a `Trie`, this is true even when
    /// the stored value is `Nil`; leaves have no keys, so return `false`.
    pub fn contains(&self, key: i32) -> bool {
        match self {
            TrieValue::Trie(trie) => trie.query(key).is_some(),
            _ => false,
        }
    }

    /// Inserts `value` at `key` into a new `TrieValue`. `Nil` promotes to a
    /// fresh trie, `Trie` passes through, and leaf variants panic.
    pub fn insert(&self, key: i32, value: impl Into<TrieValue>) -> TrieValue {
        match self {
            TrieValue::Trie(trie) => TrieValue::Trie(trie.insert(key, value)),
            TrieValue::Nil => TrieValue::Trie(SkyTrie::new().insert(key, value)),
            _ => panic!("cannot insert into a leaf value"),
        }
    }

    /// Inserts `value` at the deep path `key` into a new `TrieValue`. `Nil`
    /// promotes to a fresh trie, `Trie` passes through, and leaf variants panic.
    pub fn insert_deep<const N: usize>(&self, key: [i32; N], value: impl Into<TrieValue>) -> TrieValue {
        match self {
            TrieValue::Trie(trie) => TrieValue::Trie(trie.insert_deep(key, value)),
            TrieValue::Nil => TrieValue::Trie(SkyTrie::new().insert_deep(key, value)),
            _ => panic!("cannot insert into a leaf value"),
        }
    }
}

impl From<SkyTrie> for TrieValue {
    fn from(t: SkyTrie) -> Self {
        Self::Trie(t)
    }
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
