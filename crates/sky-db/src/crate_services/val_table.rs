use crate::Val;
use crate::internal_types::Vid;
use crate::internal_types::key::KEY_VAL_TABLE;
use crate::{QueryError, TransactError};
use sky_types::storage::MemEdit;
use sky_types::trie::*;

const VAL_TYPE_U32: u8 = 16;
const VAL_TYPE_STRING: u8 = 17;

fn bytes_from_val(val: &Val) -> Vec<u8> {
    let data_bytes = match &val {
        Val::U32(u) => &u.to_be_bytes(),
        Val::String(s) => s.as_bytes(),
    };
    let type_byte = match &val {
        Val::U32(_) => VAL_TYPE_U32,
        Val::String(_) => VAL_TYPE_STRING,
    };
    let mut bytes = vec![type_byte];
    bytes.extend_from_slice(data_bytes);
    bytes
}

fn val_from_bytes(bytes: &[u8]) -> Val {
    match bytes[0] {
        VAL_TYPE_U32 => Val::U32(u32::from_be_bytes(
            bytes[1..].try_into().expect("parse u32 from bytes"),
        )),
        VAL_TYPE_STRING => {
            Val::String(String::from_utf8(bytes[1..].to_vec()).expect("parse string from bytes"))
        }
        _ => unreachable!(),
    }
}

async fn restore_on_err(
    trie: &mut MemEdit,
    f: impl AsyncFnOnce(&mut MemEdit) -> Result<Vid, TransactError>,
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
pub async fn insert(trie: &mut MemEdit, val: Val) -> Result<Vid, TransactError> {
    let result = restore_on_err(trie, async |trie| {
        trie.descend(KEY_VAL_TABLE).await?;
        let bytes = bytes_from_val(&val);
        let mut vid = Vid::for_search(universal_hash::hash(&bytes, 0) & TrieKey::MASK);
        for _ in 0..SEARCH_SIZE {
            match trie.query(vid.to_id()).await? {
                None => {
                    trie.insert(vid.to_id(), TrieValue::Bytes(bytes)).await?;
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
    let query_val = query(trie, result).await.expect("should find val");
    debug_assert_eq!(query_val, Some(val));
    Ok(result)
}

pub async fn query<T>(trie: &T, vid: Vid) -> Result<Option<Val>, QueryError>
where
    T: QueryCursor + TrieStream + TrieSnap + TrieQuery,
{
    let mut trie = trie.snapshot();
    trie.descend(KEY_VAL_TABLE).await?;
    let value = trie.query(vid.to_id()).await?;
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
    use crate::{Val, val};
    use sky_types::storage::MemLoad;
    use sky_types::storage::load::StoreLoad;

    #[tokio::test]
    async fn insert_and_query() {
        let mut trie = MemLoad::new();
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
            let table_val = query(&trie, vid).await.expect("Failed to query");
            assert_eq!(Some(val), table_val);
        }
    }

    #[tokio::test]
    async fn negative_numbers() {
        let mut trie = MemEdit::new();
        let vid = insert(&mut trie, val(-1)).await.expect("Failed to insert");
        let table_val = query(&trie, vid).await.expect("Failed to query");
        assert_eq!(Some(val(-1)), table_val);
    }

    #[tokio::test]
    async fn same_value_inserted_twice() {
        let mut trie = MemLoad::new();
        let vid = trie
            .edit(async |trie| {
                let vid = insert(trie, val(101)).await?;
                let vid2 = insert(trie, val(101)).await?;
                assert_eq!(vid, vid2);
                Ok(vid)
            })
            .await
            .unwrap();
        let table_val = query(&trie, vid).await.expect("Failed to query");
        assert_eq!(Some(val(101)), table_val);
    }

    #[tokio::test]
    async fn string_insert_and_query() {
        let mut trie = MemLoad::new();
        let vid = trie
            .edit(async |trie| {
                let vid = insert(trie, Val::String("hello".into())).await?;
                Ok(vid)
            })
            .await
            .unwrap();
        let val = query(&trie, vid).await.expect("Failed to query");
        assert_eq!(Some(Val::String("hello".into())), val);
    }
}
