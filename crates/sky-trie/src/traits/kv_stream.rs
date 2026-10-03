use crate::services::map_base::kv_stream;
use crate::{Buffer, Snap};
use crate::{MapBase, TrieValue};
use futures::{Stream, StreamExt};

pub trait KvStream {
    /// Stream key values from the trie
    fn kv_stream(&self) -> impl Stream<Item = (i32, TrieValue)>;

    /// Stream sub-trie values from the trie.
    fn map_base_stream(&self) -> impl Stream<Item = (i32, MapBase)>;

    /// Stream u32 values from the trie.
    fn u32_stream(&self) -> impl Stream<Item = (i32, u32)>;
}

impl<T: Snap<Snapshot: Buffer> + Buffer> KvStream for T {
    fn kv_stream(&self) -> impl Stream<Item = (i32, TrieValue)> {
        kv_stream(self.get_root(), self.snapshot())
    }
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
}
