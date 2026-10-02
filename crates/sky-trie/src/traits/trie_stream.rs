use crate::services::map_base::kv_stream;
use crate::{Buffer, TrieSnap};
use crate::{MapBase, TrieValue};
use futures::{Stream, StreamExt};

pub trait TrieStream {
    /// Stream sub-trie values from the trie.
    fn map_base_stream(&self) -> impl Stream<Item = (i32, MapBase)>;

    /// Stream u32 values from the trie.
    fn u32_stream(&self) -> impl Stream<Item = (i32, u32)>;

    /// Stream key values from the trie
    fn kv_stream(&self) -> impl Stream<Item = (i32, TrieValue)>;
}

impl<T: TrieSnap<Snapshot: Buffer> + Buffer> TrieStream for T {
    fn map_base_stream(&self) -> impl Stream<Item = (i32, MapBase)> {
        kv_stream(self.get_root(), self.snapshot()).filter_map(move |(key, value)| async move {
            if let TrieValue::SubTrie(map_base) = value {
                Some((key, map_base))
            } else {
                None
            }
        })
    }
    fn u32_stream(&self) -> impl Stream<Item = (i32, u32)> {
        kv_stream(self.get_root(), self.snapshot()).filter_map(|(key, value)| async move {
            if let TrieValue::U32(val) = value {
                Some((key, val))
            } else {
                None
            }
        })
    }
    fn kv_stream(&self) -> impl Stream<Item = (i32, TrieValue)> {
        kv_stream(self.get_root(), self.snapshot())
    }
}
