use crate::_internal::KEY_VAL_TABLE;
use crate::_internal::Vid;
use crate::Val;
use crate::trie::*;
use crate::trie::SkyTrieMut;
use crate::{QueryError, TransactError};

const VAL_TYPE_U32: u8 = 16;
const VAL_TYPE_STRING: u8 = 17;
const VAL_TYPE_U64: u8 = 18;
const VAL_TYPE_BYTES: u8 = 19;

fn bytes_from_val(val: &Val) -> Vec<u8> {
    match val {
        Val::U32(u) => {
            let mut b = vec![VAL_TYPE_U32];
            b.extend_from_slice(&u.to_be_bytes());
            b
        }
        Val::U64(u) => {
            let mut b = vec![VAL_TYPE_U64];
            b.extend_from_slice(&u.to_be_bytes());
            b
        }
        Val::String(s) => {
            let mut b = vec![VAL_TYPE_STRING];
            b.extend_from_slice(s.as_bytes());
            b
        }
        Val::Bytes(bs) => {
            let mut b = vec![VAL_TYPE_BYTES];
            b.extend_from_slice(bs);
            b
        }
    }
}

fn val_from_bytes(bytes: &[u8]) -> Val {
    match bytes[0] {
        VAL_TYPE_U32 => Val::U32(u32::from_be_bytes(
            bytes[1..5].try_into().expect("parse u32 from bytes"),
        )),
        VAL_TYPE_U64 => Val::U64(u64::from_be_bytes(
            bytes[1..9].try_into().expect("parse u64 from bytes"),
        )),
        VAL_TYPE_STRING => {
            Val::String(String::from_utf8(bytes[1..].to_vec()).expect("parse string from bytes"))
        }
        VAL_TYPE_BYTES => Val::Bytes(bytes[1..].to_vec()),
        _ => unreachable!(),
    }
}

async fn restore_on_err(
    trie: &mut SkyTrieMut,
    f: impl AsyncFnOnce(&mut SkyTrieMut) -> Result<Vid, TransactError>,
) -> Result<Vid, TransactError> {
    let start = trie.backup();
    match f(trie).await {
        Ok(vid) => {
            // Do not attempt using `restore` here to avoid doing
            // ascends in `f`. Doing so would also erase any work that
            // was accomplished in `f`.
            Ok(vid)
        }
        Err(e) => {
            trie.restore(start);
            Err(e)
        }
    }
}

const SEARCH_SIZE: usize = 4000;
pub async fn insert(trie: &mut SkyTrieMut, val: Val) -> Result<Vid, TransactError> {
    let result = restore_on_err(trie, async |trie| {
        trie.descend(KEY_VAL_TABLE);
        let bytes = bytes_from_val(&val);
        let mut vid = Vid::for_search(universal_hash::hash(&bytes, 0) & TrieKey::MASK);
        for _ in 0..SEARCH_SIZE {
            match trie.query(vid.to_id()) {
                None => {
                    trie.insert(vid.to_id(), TrieValue::Bytes(bytes)).await;
                    trie.ascend();
                    return Ok(vid);
                }
                Some(TrieValue::Bytes(existing)) if &existing == &bytes => {
                    trie.ascend();
                    return Ok(vid);
                }
                Some(_) => vid = vid.next_search(),
            }
        }
        Err(TransactError::NoSpaceInValueTable)
    })
    .await?;
    let query_val = query(trie, result).expect("should find val");
    debug_assert_eq!(query_val, Some(val));
    Ok(result)
}

pub fn query<T>(trie: &T, vid: Vid) -> Result<Option<Val>, QueryError>
where
    T: QueryCursor + Snap + Query,
{
    let mut trie = trie.snapshot();
    trie.descend(KEY_VAL_TABLE);
    let value = trie.query(vid.to_id());
    let val = if let Some(TrieValue::Bytes(bytes)) = value {
        let val = val_from_bytes(&bytes);
        Some(val)
    } else {
        None
    };
    Ok(val)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trie::SkyTrie;
    use crate::{Val, val};

    #[tokio::test]
    async fn insert_and_query() {
        let mut trie = SkyTrie::new();
        let (vids, vals) = trie
            .edit(async |trie| {
                let mut vids = Vec::new();
                let mut vals = Vec::new();
                for i in 0..100 {
                    let i_val = val(i);
                    vals.push(i_val.clone());
                    let vid = insert(trie, i_val).await?;
                    vids.push(vid);
                }
                Ok((vids, vals))
            })
            .await
            .unwrap();
        for (vid, val) in vids.into_iter().zip(vals) {
            let table_val = query(&trie, vid).expect("Failed to query");
            assert_eq!(Some(val), table_val);
        }
    }

    #[tokio::test]
    async fn negative_numbers() {
        let mut trie = SkyTrieMut::new();
        let vid = insert(&mut trie, val(-1)).await.expect("Failed to insert");
        let table_val = query(&trie, vid).expect("Failed to query");
        assert_eq!(Some(val(-1)), table_val);
    }

    #[tokio::test]
    async fn same_value_inserted_twice() {
        let mut trie = SkyTrie::new();
        let vid = trie
            .edit(async |trie| {
                let vid = insert(trie, val(101)).await?;
                let vid2 = insert(trie, val(101)).await?;
                assert_eq!(vid, vid2);
                Ok(vid)
            })
            .await
            .unwrap();
        let table_val = query(&trie, vid).expect("Failed to query");
        assert_eq!(Some(val(101)), table_val);
    }

    #[tokio::test]
    async fn string_insert_and_query() {
        let mut trie = SkyTrie::new();
        let vid = trie
            .edit(async |trie| {
                let vid = insert(trie, Val::String("hello".into())).await?;
                Ok(vid)
            })
            .await
            .unwrap();
        let val = query(&trie, vid).expect("Failed to query");
        assert_eq!(Some(Val::String("hello".into())), val);
    }

    #[tokio::test]
    async fn u64_insert_and_query() {
        let mut trie = SkyTrie::new();
        let vid = trie
            .edit(async |trie| {
                let vid = insert(trie, Val::U64(u64::MAX)).await?;
                Ok(vid)
            })
            .await
            .unwrap();
        let val = query(&trie, vid).expect("Failed to query");
        assert_eq!(Some(Val::U64(u64::MAX)), val);
    }

    #[tokio::test]
    async fn bytes_insert_and_query() {
        let mut trie = SkyTrie::new();
        let (vid, vid_empty) = trie
            .edit(async |trie| {
                let vid = insert(trie, Val::Bytes(vec![1, 2, 3, 4])).await?;
                let vid_empty = insert(trie, Val::Bytes(vec![])).await?;
                Ok((vid, vid_empty))
            })
            .await
            .unwrap();
        let val = query(&trie, vid).expect("Failed to query");
        assert_eq!(Some(Val::Bytes(vec![1, 2, 3, 4])), val);
        let val_empty = query(&trie, vid_empty).expect("Failed to query");
        assert_eq!(Some(Val::Bytes(vec![])), val_empty);
    }

    #[tokio::test]
    async fn distinct_types_get_distinct_vids() {
        let mut trie = SkyTrie::new();
        let (u32_vid, u64_vid, bytes_vid) = trie
            .edit(async |trie| {
                let u32_vid = insert(trie, Val::U32(5)).await?;
                let u64_vid = insert(trie, Val::U64(5)).await?;
                let bytes_vid = insert(trie, Val::Bytes(vec![5])).await?;
                Ok((u32_vid, u64_vid, bytes_vid))
            })
            .await
            .unwrap();
        assert_ne!(u32_vid, u64_vid);
        assert_ne!(u32_vid, bytes_vid);
        assert_ne!(u64_vid, bytes_vid);
    }
}
