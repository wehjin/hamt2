use crate::trie::{MapBase, TrieStream, TrieValue};
use futures::{Stream, StreamExt};

/// Implement this trait and provide `to_subtrie` to acquire streaming access
/// to stored values.
pub trait TrieWalk: TrieStream {
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
}