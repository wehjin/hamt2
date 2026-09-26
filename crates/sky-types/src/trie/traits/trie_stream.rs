use crate::trie::map_base::kv_stream;
use crate::trie::{BaseRead, TrieSnap};
use crate::trie::{MapBase, TrieValue};
use futures::{Stream, StreamExt};

/// Implement this trait and provide `to_subtrie` to acquire streaming access
/// to stored values.
pub trait TrieWalk: TrieSnap<Snapshot: BaseRead> + BaseRead {
    type Subtrie: TrieWalk;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie;

    fn to_subtrie_in_value(&self, trie_value: TrieValue) -> Option<Self::Subtrie> {
        match trie_value {
            TrieValue::U32(_) => None,
            TrieValue::SubTrie(map_base) => Some(self.to_subtrie(map_base)),
        }
    }

    fn subtrie_stream(&self) -> impl Stream<Item = (i32, Self::Subtrie)> {
        self.map_base_stream()
            .map(|(key, map_base)| (key, self.to_subtrie(map_base)))
    }

    /// Stream sub-trie values from the trie.
    fn map_base_stream(&self) -> impl Stream<Item = (i32, MapBase)> {
        kv_stream(self.read_root(), self.snapshot()).filter_map(move |(key, value)| async move {
            if let TrieValue::SubTrie(map_base) = value {
                Some((key, map_base))
            } else {
                None
            }
        })
    }

    /// Stream u32 values from the trie.
    fn u32_stream(&self) -> impl Stream<Item = (i32, u32)> {
        kv_stream(self.read_root(), self.snapshot()).filter_map(|(key, value)| async move {
            if let TrieValue::U32(val) = value {
                Some((key, val))
            } else {
                None
            }
        })
    }
}
