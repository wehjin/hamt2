use crate::{HashKey, MapBase, TrieValue, map_base};

/// An immutable, persistent trie. `Clone` is a cheap snapshot (structural
/// sharing via `Arc`), and every insert returns a new trie, leaving the
/// original unchanged.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SkyTrie {
    pub(crate) root: MapBase,
}

impl Default for SkyTrie {
    fn default() -> Self {
        Self::new()
    }
}

impl SkyTrie {
    pub fn new() -> Self {
        Self {
            root: MapBase::empty(),
        }
    }

    pub fn list_keys(&self) -> Vec<i32> {
        self.clone().into_iter().map(|(key, _)| key).collect()
    }

    /// Inserts `value` at `key`, returning a new trie. The original is unchanged.
    pub fn insert(&self, key: i32, value: impl Into<TrieValue>) -> SkyTrie {
        let root = map_base::insert_kv(self.root.clone(), HashKey::new(key), value.into());
        SkyTrie { root }
    }

    /// Inserts `value` at the deep path `key`, returning a new trie. The original is unchanged.
    pub fn insert_deep<const N: usize>(&self, key: [i32; N], value: impl Into<TrieValue>) -> SkyTrie {
        let root = map_base::insert_kv_deep(self.root.clone(), key, value.into());
        SkyTrie { root }
    }

    /// Returns the value stored at `key`, or `None` if absent.
    pub fn query(&self, key: i32) -> Option<TrieValue> {
        map_base::query_value(self.root.clone(), HashKey::new(key))
    }

    /// Returns the u32 value stored at `key`, or `None` if absent or not a u32.
    pub fn query_u32(&self, key: i32) -> Option<u32> {
        if let Some(TrieValue::U32(value)) = self.query(key) {
            Some(value)
        } else {
            None
        }
    }

    /// Returns all keys and values in this trie.
    pub fn query_all(&self) -> Vec<(i32, TrieValue)> {
        map_base::query_keys_values(self.root.clone())
    }

    /// Returns the value stored at the deep path `key`, or `None` if absent.
    pub fn query_deep<const N: usize>(&self, key: [i32; N]) -> Option<TrieValue> {
        map_base::query_value_deep(self.root.clone(), key)
    }
}

#[cfg(test)]
mod tests {
    use crate::{SkyTrie, TrieValue};

    #[test]
    fn insert_returns_new_trie() {
        let trie = SkyTrie::new();
        let next = trie.insert(7, TrieValue::U32(7));
        assert_eq!(trie.query(7), None);
        assert_eq!(next.query(7), Some(TrieValue::U32(7)));
    }

    #[test]
    fn insert_deep_returns_new_trie() {
        let trie = SkyTrie::new();
        let next = trie.insert_deep([1, 2, 3], TrieValue::U32(42));
        assert_eq!(trie.query_deep([1, 2, 3]), None);
        assert_eq!(next.query_deep([1, 2, 3]), Some(TrieValue::U32(42)));
    }

    #[test]
    fn query_deep_returns_trie() {
        let trie = SkyTrie::new().insert_deep([1, 2, 3], TrieValue::U32(42));
        let sub = match trie.query_deep([1, 2]) {
            Some(TrieValue::Trie(t)) => t,
            _ => panic!("expected trie"),
        };
        assert_eq!(sub.query(3), Some(TrieValue::U32(42)));
        assert_eq!(trie.query_deep([1, 9]), None);
    }
}
