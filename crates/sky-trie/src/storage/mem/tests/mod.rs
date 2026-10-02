use crate::storage::MemView;
use crate::storage::mem::tests::fixtures::{
    edit_assert_trie, error_edit_assert_trie, start_assert_trie,
};
use crate::trie::{TrieInsert, TrieQuery, TrieSnap, TrieStream, TrieValue};
use futures::StreamExt;

#[cfg(test)]
mod edit_walks;

#[tokio::test]
async fn view_has_query() {
    let mem = MemView::new().snapshot();
    let values = mem.query_all().await;
    assert_eq!(values, vec![]);
}

#[tokio::test]
async fn view_has_snap() {
    let mem = MemView::new().snapshot();
    let values = mem.snapshot().query_all().await;
    assert_eq!(values, vec![]);
}

#[tokio::test]
async fn view_has_stream() {
    let mem = MemView::new().snapshot();
    let values = mem.u32_stream().collect::<Vec<_>>().await;
    assert_eq!(values, vec![]);
}

#[tokio::test]
async fn edit_inserts() {
    let insert_value = TrieValue::U32(34);
    let mut mem = MemView::new();
    mem.edit(async |edit| {
        edit.insert(33, insert_value.clone()).await?;
        Ok(())
    })
    .await
    .unwrap();
    let query_value = mem.query(33).await.unwrap();
    assert_eq!(query_value, insert_value)
}

#[tokio::test]
async fn main_accumulates_ok_edit() {
    let mut trie = start_assert_trie().await;
    edit_assert_trie(&mut trie, 62).await;
    let values = trie.query_all().await;
    assert_eq!(values, vec![(62, TrieValue::U32(63))]);
}

#[tokio::test]
async fn main_rejects_err_edit() {
    let mut trie = start_assert_trie().await;
    error_edit_assert_trie(&mut trie, 62).await;
    let values = trie.query_all().await;
    assert_eq!(values.len(), 0);
}

#[tokio::test]
async fn main_accumulates_second_ok_edit() {
    let mut trie = start_assert_trie().await;
    edit_assert_trie(&mut trie, 20).await;
    edit_assert_trie(&mut trie, 30).await;
    let mut values = trie.query_all().await;
    values.sort_by(|(a, _), (b, _)| a.cmp(b));
    assert_eq!(
        values,
        vec![(20, TrieValue::U32(21)), (30, TrieValue::U32(31)),]
    );
}

#[tokio::test]
async fn main_rejects_second_err_edit() {
    let mut trie = start_assert_trie().await;
    edit_assert_trie(&mut trie, 20).await;
    error_edit_assert_trie(&mut trie, 30).await;
    let mut values = trie.query_all().await;
    values.sort_by(|(a, _), (b, _)| a.cmp(b));
    assert_eq!(values, vec![(20, TrieValue::U32(21))]);
}

mod fixtures {
    use crate::storage::MemView;
    use crate::trie::{TrieInsert, TrieQuery, TrieValue};
    use anyhow::anyhow;

    pub async fn start_assert_trie() -> MemView {
        let trie = MemView::new();
        let values = trie.query_all().await;
        assert_eq!(values.len(), 0);
        trie
    }

    pub async fn edit_assert_trie(trie: &mut MemView, tag: i32) {
        let out = trie
            .edit(async |edit| {
                let value = TrieValue::U32(tag as u32 + 1);
                edit.insert(tag, value.clone()).await?;
                let query = edit.query(tag).await;
                assert_eq!(query, Some(value));
                Ok(tag + 2)
            })
            .await
            .unwrap();
        assert_eq!(out, tag + 2);
    }

    pub async fn error_edit_assert_trie(trie: &mut MemView, tag: i32) {
        let out = trie
            .edit::<_, ()>(async |edit| {
                let value = TrieValue::U32(tag as u32 + 1);
                edit.insert(tag, value.clone()).await?;
                let query = edit.query(tag).await;
                assert_eq!(query, Some(value));
                Err(anyhow!("trouble in edit"))
            })
            .await;
        assert!(out.is_err());
    }
}
