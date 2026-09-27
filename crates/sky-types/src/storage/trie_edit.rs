#[cfg(test)]
use crate::storage::MemEdit;
use crate::storage::edit::StoreEdit;
use crate::trie::{
    MapBase, TrieInsert, TrieInsertError, TrieQuery, TrieQueryError, TrieSnap, TrieStream,
    TrieValue, TrieWalk,
};
use futures::Stream;
#[cfg(test)]
pub fn mem_edit_new() -> TrieEdit<MemEdit> {
    let inner = MemEdit::new();
    TrieEdit { inner }
}

#[derive(Debug)]
pub struct TrieEdit<S: StoreEdit> {
    pub(crate) inner: S,
}

impl<S: StoreEdit> StoreEdit for TrieEdit<S> {}

impl<S: StoreEdit> TrieInsert for TrieEdit<S> {
    async fn insert(
        &mut self,
        key: i32,
        value: impl Into<TrieValue>,
    ) -> Result<&mut Self, TrieInsertError> {
        let _ = self.inner.insert(key, value).await?;
        Ok(self)
    }

    async fn deep_insert<const N: usize>(
        &mut self,
        key: [i32; N],
        value: impl Into<TrieValue>,
        replace_tail: bool,
    ) -> Result<&mut Self, TrieInsertError> {
        let _ = self.inner.deep_insert(key, value, replace_tail).await?;
        Ok(self)
    }
}
impl<S: StoreEdit> TrieWalk for TrieEdit<S> {
    type Subtrie = Self;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        Self {
            inner: self.inner.to_subtrie(subtrie_root),
        }
    }
}
impl<S: StoreEdit> TrieStream for TrieEdit<S> {
    fn map_base_stream(&self) -> impl Stream<Item = (i32, MapBase)> {
        self.inner.map_base_stream()
    }

    fn u32_stream(&self) -> impl Stream<Item = (i32, u32)> {
        self.inner.u32_stream()
    }
}
impl<S: StoreEdit> TrieSnap for TrieEdit<S> {
    type Snapshot = Self;

    fn snapshot(&self) -> Self::Snapshot {
        Self {
            inner: self.inner.snapshot(),
        }
    }
}
impl<S: StoreEdit> TrieQuery for TrieEdit<S> {
    async fn query(&self, key: i32) -> Result<Option<TrieValue>, TrieQueryError> {
        self.inner.query(key).await
    }

    async fn query_u32(&self, key: i32) -> Result<Option<u32>, TrieQueryError> {
        self.inner.query_u32(key).await
    }

    async fn query_all(&self) -> Result<Vec<(i32, TrieValue)>, TrieQueryError> {
        self.inner.query_all().await
    }

    async fn deep_query<const N: usize>(
        &self,
        key: [i32; N],
    ) -> Result<Option<TrieValue>, TrieQueryError> {
        self.inner.deep_query(key).await
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::mem_edit_new;
    use crate::trie::{TrieInsert, TrieQuery, TrieValue};

    #[tokio::test]
    async fn query_value_exists_works() {
        let mut m = mem_edit_new();
        m.insert(32, TrieValue::U32(33)).await.unwrap();
        let v = m.query(32).await.unwrap();
        assert_eq!(v, Some(TrieValue::U32(33)));
    }

    #[tokio::test]
    async fn deep_insert_and_query_works() {
        let mut m = mem_edit_new();
        m.deep_insert([1, 2, 3], 45, false).await.unwrap();
        let v = m.deep_query([1, 2, 3]).await.unwrap();
        assert_eq!(v, Some(TrieValue::U32(45)));
    }
}
