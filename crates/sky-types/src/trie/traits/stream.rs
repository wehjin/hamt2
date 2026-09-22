use crate::storage::ReadStorage;
use crate::trie::map_base::kv_stream;
use crate::trie::{MapBase, TrieValue};
use futures::{Stream, StreamExt};

/// Implement this trait and provide `to_subtrie` to acquire streaming access
/// to stored values.
pub trait TrieStream: ReadStorage {
    type Subtrie: TrieStream;

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

    /// Stream all map-base key values in the trie.
    fn map_base_stream(&self) -> impl Stream<Item = (i32, MapBase)> {
        let stream = kv_stream(self.read_root(), self.snapshot());
        stream.filter_map(move |(key, value)| async move {
            if let TrieValue::SubTrie(map_base) = value {
                Some((key, map_base))
            } else {
                None
            }
        })
    }

    /// Stream all u32 keyed values in the trie.
    fn u32_stream(&self) -> impl Stream<Item = (i32, u32)> {
        let stream = kv_stream(self.read_root(), self.snapshot());
        stream.filter_map(|(key, value)| async move {
            if let TrieValue::U32(val) = value {
                Some((key, val))
            } else {
                None
            }
        })
    }
}
