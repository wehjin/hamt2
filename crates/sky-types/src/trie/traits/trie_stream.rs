use crate::trie::map_base::kv_stream;
use crate::trie::{BaseRead, TrieSnap};
use crate::trie::{MapBase, TrieValue};
use futures::{Stream, StreamExt};

pub trait TrieStream {
    /// Stream sub-trie values from the trie.
    fn map_base_stream(&self) -> impl Stream<Item = (i32, MapBase)>;

    /// Stream u32 values from the trie.
    fn u32_stream(&self) -> impl Stream<Item = (i32, u32)>;
}

impl<T: TrieSnap<Snapshot: BaseRead> + BaseRead> TrieStream for T {
    fn map_base_stream(&self) -> impl Stream<Item = (i32, MapBase)> {
        kv_stream(self.read_root(), self.snapshot()).filter_map(move |(key, value)| async move {
            if let TrieValue::SubTrie(map_base) = value {
                Some((key, map_base))
            } else {
                None
            }
        })
    }
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
