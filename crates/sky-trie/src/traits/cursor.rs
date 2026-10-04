use crate::{CursorPos, Insert, InsertOption, MapBase, Query, TrieKey, TrieValue};

pub trait QueryCursor: Query {
    fn cursor_pos(&self) -> &CursorPos;

    fn cursor_pos_mut(&mut self) -> &mut CursorPos;

    /// Returns the root of the trie
    fn top_root(&self) -> MapBase {
        self.cursor_pos().top_root()
    }

    /// Moves up one level in the trie.
    fn ascend(&mut self) -> Option<(TrieKey, MapBase)> {
        self.cursor_pos_mut().ascend()
    }

    /// Moves up n levels in the trie.
    fn ascend_n(&mut self, n: usize) {
        for _ in 0..n {
            self.ascend();
        }
    }

    /// Moves down one level in the trie.
    fn descend(&mut self, key: impl Into<TrieKey>) {
        let key = key.into();
        let lower_root = match self.query(key.into()) {
            None => MapBase::empty(),
            Some(TrieValue::U32(_)) => panic!("key is occupied by a u32 value"),
            Some(TrieValue::Bytes(_)) => panic!("key is occupied by a bytes value"),
            Some(TrieValue::SubTrie(lower_root)) => lower_root,
        };
        self.cursor_pos_mut().descend(key, lower_root);
    }

    /// Moves down n levels in the trie.
    fn descend_n(&mut self, keys: impl IntoIterator<Item = impl Into<TrieKey>>) -> usize {
        let mut count = 0;
        for key in keys {
            self.descend(key);
            count += 1;
        }
        count
    }

    /// Provides the cursor position for later restoration.
    fn backup(&self) -> CursorPos {
        self.cursor_pos().clone()
    }

    /// Restores cursor position to a previous backup.
    fn restore(&mut self, pos: CursorPos) {
        *self.cursor_pos_mut() = pos;
    }
}

#[allow(async_fn_in_trait)]
pub trait InsertCursor: QueryCursor + Insert {
    /// Inserts `value` into the trie after descending the path marked by `keys`.
    async fn insert_deep(
        &mut self,
        keys: impl IntoIterator<Item = impl Into<TrieKey>>,
        value: impl Into<TrieValue>,
        delete_others: bool,
    ) -> &mut Self {
        let (descend_keys, insert_key) = {
            let mut keys = keys.into_iter().map(|k| k.into()).collect::<Vec<_>>();
            let last_key = keys.pop().expect("too few keys");
            (keys, last_key)
        };
        let count = self.descend_n(descend_keys);
        let insert_options = if delete_others {
            vec![InsertOption::DeleteOthers]
        } else {
            vec![]
        };
        self.insert_with_options(insert_key.into(), value, insert_options)
            .await;
        self.ascend_n(count);
        self
    }
}
