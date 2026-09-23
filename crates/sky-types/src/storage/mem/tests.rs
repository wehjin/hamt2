use crate::storage::mem::tests::fixtures::{edit_assert_trie, start_assert_trie};
use crate::trie::{TrieInsert, TrieQuery, TrieValue};
use anyhow::anyhow;

#[tokio::test]
async fn main_accumulates_edit() {
    let mut trie = start_assert_trie().await;
    edit_assert_trie(&mut trie, 62).await;
    let values = trie.query_all().await.unwrap();
    assert_eq!(values, vec![(62, TrieValue::U32(63))]);
}

#[tokio::test]
async fn main_rejects_edit_error() {
    let mut trie = start_assert_trie().await;
    let out = trie
        .edit::<_, ()>(async |edit| {
            edit.insert(62, TrieValue::U32(63)).await?;
            Err(anyhow!("trouble in edit"))
        })
        .await;
    assert!(out.is_err());

    let values = trie.query_all().await.unwrap();
    assert_eq!(values.len(), 0);
}

#[tokio::test]
async fn main_accumulates_second_edit() {
    let mut trie = start_assert_trie().await;
    edit_assert_trie(&mut trie, 20).await;
    edit_assert_trie(&mut trie, 30).await;
    let mut values = trie.query_all().await.unwrap();
    values.sort_by(|(a, _), (b, _)| a.cmp(b));
    assert_eq!(
        values,
        vec![(20, TrieValue::U32(21)), (30, TrieValue::U32(31)),]
    );
}

mod fixtures {
    use crate::storage::MemTrie;
    use crate::trie::{TrieInsert, TrieQuery, TrieValue};

    pub async fn start_assert_trie() -> MemTrie {
        let trie = MemTrie::new();
        let values = trie.query_all().await.unwrap();
        assert_eq!(values.len(), 0);
        trie
    }

    pub async fn edit_assert_trie(trie: &mut MemTrie, tag: i32) {
        let out = trie
            .edit(async |edit| {
                let value = TrieValue::U32(tag as u32 + 1);
                edit.insert(tag, value).await?;
                let query = edit.query(tag).await?;
                assert_eq!(query, Some(value));
                Ok(tag + 2)
            })
            .await
            .unwrap();
        assert_eq!(out, tag + 2);
    }
}
