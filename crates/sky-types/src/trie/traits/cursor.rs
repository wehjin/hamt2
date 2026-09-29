use crate::trie::{
    CursorPos, MapBase, TrieInsert, TrieInsertError, TrieKey, TrieQueryError, TrieValue,
};

#[allow(async_fn_in_trait)]
pub trait QueryCursor {
    fn top_root(&self) -> MapBase;

    fn ascend(&mut self) -> Option<(TrieKey, MapBase)>;
    fn ascend_n(&mut self, n: usize) {
        for _ in 0..n {
            self.ascend();
        }
    }
    async fn descend(&mut self, key: impl Into<TrieKey>) -> Result<(), TrieQueryError>;
    async fn descend_keys(
        &mut self,
        keys: impl IntoIterator<Item = impl Into<TrieKey>>,
    ) -> Result<usize, TrieQueryError> {
        let mut count = 0;
        for key in keys {
            self.descend(key).await?;
            count += 1;
        }
        Ok(count)
    }

    /// Provides the cursor position for later restoration.
    fn backup(&self) -> CursorPos;

    /// Restores cursor position to a previous backup.
    fn restore(&mut self, pos: CursorPos);
}

#[allow(async_fn_in_trait)]
pub trait InsertCursor: QueryCursor + TrieInsert {
    async fn insert_deep(
        &mut self,
        keys: impl IntoIterator<Item = impl Into<TrieKey>>,
        value: impl Into<TrieValue>,
        replace_tail: bool,
    ) -> Result<&mut Self, TrieInsertError> {
        let start_pos = self.backup();
        let (descend_keys, insert_key) = {
            let mut keys = keys.into_iter().map(|k| k.into()).collect::<Vec<_>>();
            let last_key = keys.pop().expect("too few keys");
            (keys, last_key)
        };
        let count = match self.descend_keys(descend_keys).await {
            Ok(count) => count,
            Err(e) => {
                self.restore(start_pos);
                return Err(TrieInsertError::TrieQuery(e));
            }
        };
        let result = if replace_tail {
            self.clear_insert(insert_key.into(), value).await
        } else {
            self.insert(insert_key.into(), value).await
        };
        if let Err(e) = result {
            self.restore(start_pos);
            return Err(e);
        }
        self.ascend_n(count);
        Ok(self)
    }
}
