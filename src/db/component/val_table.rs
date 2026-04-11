use crate::db::component::key::KEY_VAL_TABLE;
use crate::db::component::u32;
use crate::db::{Val, Vid};
use crate::space::Space;
use crate::trie::mem::value::MemValue;
use crate::trie::SpaceTrie;
use crate::{hash, QueryError, TransactError};
use redb::Value;

pub async fn insert<T: Space>(
    trie: SpaceTrie<T>,
    val: Val,
) -> Result<(SpaceTrie<T>, Vid), TransactError> {
    let bytes = match &val {
        Val::U32(u) => &u.to_be_bytes(),
        Val::String(s) => s.as_bytes(),
    };
    let bytes_type = match &val {
        Val::U32(_) => VAL_TYPE_U32,
        Val::String(_) => VAL_TYPE_STRING,
    };

    let mut hash = (hash::universal(bytes, 1) & 0x7FFFFFFF) as i32;
    for _ in 0..1000 {
        let hash_trie = find_hash_trie(&trie, hash).await?;
        match hash_trie {
            None => {
                let trie = insert_bytes(trie, hash, bytes, bytes_type).await?;
                return Ok((trie, Vid::from_id(hash)));
            }
            Some(bytes_trie) => {
                if is_equal_bytes(&bytes_trie, bytes, bytes_type).await? {
                    return Ok((trie, Vid::from_id(hash)));
                }
                if hash == i32::MAX {
                    hash = 0;
                } else {
                    hash += 1;
                }
            }
        }
    }
    Err(TransactError::NoSpaceInValueTable)
}

pub async fn query<T: Space>(trie: &SpaceTrie<T>, vid: Vid) -> Result<Option<Val>, QueryError> {
    match find_hash_trie(trie, vid.to_id()).await? {
        None => Ok(None),
        Some(val_trie) => {
            let Some(MemValue::U32(bytes_len)) = val_trie.query_value(SUBKEY_LEN).await? else {
                panic!("Unexpected MemValue variant")
            };
            let Some(MemValue::U32(val_type)) = val_trie.query_value(SUBKEY_VAL_TYPE).await? else {
                panic!("Unexpected MemValue variant")
            };
            let builder = u32::Read::new(&val_trie, bytes_len as usize, SUBKEY_BYTES);
            let bytes = builder.into_bytes().await;
            match val_type as u8 {
                VAL_TYPE_U32 => {
                    let v = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                    Ok(Some(Val::U32(v)))
                }
                VAL_TYPE_STRING => {
                    let s = String::from_bytes(&bytes);
                    Ok(Some(Val::String(s)))
                }
                _ => unreachable!("Invalid val_type: {:?}", val_type),
            }
        }
    }
}

const SUBKEY_LEN: i32 = 0;
const SUBKEY_VAL_TYPE: i32 = 1;
const SUBKEY_BYTES: i32 = 100;
const VAL_TYPE_U32: u8 = 0;
const VAL_TYPE_STRING: u8 = 1;

async fn insert_bytes<T: Space>(
    mut trie: SpaceTrie<T>,
    hash: i32,
    bytes: &[u8],
    bytes_type: u8,
) -> Result<SpaceTrie<T>, TransactError> {
    let u32_stream = u32::Stream::new(bytes, SUBKEY_BYTES);
    for (u32_subkey, u32_value) in u32_stream {
        trie = trie
            .deep_insert(
                [KEY_VAL_TABLE, hash, u32_subkey],
                MemValue::U32(u32_value),
                false,
            )
            .await?;
    }
    trie = trie
        .deep_insert(
            [KEY_VAL_TABLE, hash, SUBKEY_LEN],
            MemValue::U32(bytes.len() as u32),
            false,
        )
        .await?;
    trie = trie
        .deep_insert(
            [KEY_VAL_TABLE, hash, SUBKEY_VAL_TYPE],
            MemValue::U32(bytes_type as u32),
            false,
        )
        .await?;
    Ok(trie)
}

async fn is_equal_bytes<T: Space>(
    hash_trie: &SpaceTrie<T>,
    bytes: &[u8],
    bytes_type: u8,
) -> Result<bool, QueryError> {
    let Some(MemValue::U32(len)) = hash_trie.query_value(SUBKEY_LEN).await? else {
        panic!("Unexpected MemValue variant")
    };
    if len as usize != bytes.len() {
        return Ok(false);
    }
    let Some(MemValue::U32(val_type)) = hash_trie.query_value(SUBKEY_VAL_TYPE).await? else {
        panic!("Unexpected MemValue variant")
    };
    if val_type as u8 != bytes_type {
        return Ok(false);
    }
    let mut u32_stream = u32::Stream::new(bytes, SUBKEY_BYTES);
    loop {
        match u32_stream.next() {
            Some((u32_subkey, u32_value)) => {
                let saved = hash_trie.query_value(u32_subkey).await?;
                match saved {
                    None => return Ok(false),
                    Some(saved_mem_value) => {
                        let MemValue::U32(saved_u32) = saved_mem_value else {
                            panic!("Unexpected MemValue variant")
                        };
                        if u32_value != saved_u32 {
                            return Ok(false);
                        }
                        // Values match so continue to the next key.
                    }
                }
            }
            None => {
                return Ok(true);
            }
        }
    }
}

async fn find_hash_trie<T: Space>(
    trie: &SpaceTrie<T>,
    hash: i32,
) -> Result<Option<SpaceTrie<T>>, QueryError> {
    let key = [KEY_VAL_TABLE, hash];
    match trie.deep_query_value(key).await? {
        None => Ok(None),
        Some(mem_value) => {
            let bytes_trie = trie.to_subtrie_from_value(mem_value).await?;
            Ok(Some(bytes_trie))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::val;
    use crate::space::mem::MemSpace;

    #[tokio::test]
    async fn insert_and_query() {
        let space = MemSpace::new();
        let mut trie = SpaceTrie::connect(&space)
            .await
            .expect("Failed to connect to MemSpace");
        let mut vids = Vec::new();
        let mut vals = Vec::new();
        for i in 0..100 {
            let i_val = val(i);
            vals.push(i_val.clone());
            let (new_trie, vid) = insert(trie, i_val).await.expect("Failed to insert");
            trie = new_trie;
            vids.push(vid);
        }
        for (vid, val) in vids.into_iter().zip(vals) {
            let table_val = query(&trie, vid).await.expect("Failed to query");
            assert_eq!(Some(val), table_val);
        }
    }

    #[tokio::test]
    async fn negative_numbers() {
        let space = MemSpace::new();
        let trie = SpaceTrie::connect(&space)
            .await
            .expect("Failed to connect to MemSpace");
        let (trie, vid) = insert(trie, val(-1)).await.expect("Failed to insert");
        let table_val = query(&trie, vid).await.expect("Failed to query");
        assert_eq!(Some(val(-1)), table_val);
    }

    #[tokio::test]
    async fn same_value_inserted_twice() {
        let space = MemSpace::new();
        let trie = SpaceTrie::connect(&space)
            .await
            .expect("Failed to connect to MemSpace");

        let (trie, vid) = insert(trie, val(101)).await.expect("Failed to insert");
        let (trie, vid2) = insert(trie, val(101)).await.expect("Failed to insert");
        assert_eq!(vid, vid2);
        let table_val = query(&trie, vid).await.expect("Failed to query");
        assert_eq!(Some(val(101)), table_val);
    }

    #[tokio::test]
    async fn string_insert_and_query() {
        let space = MemSpace::new();
        let trie = SpaceTrie::connect(&space)
            .await
            .expect("Failed to connect to MemSpace");
        let (trie, vid) = insert(trie, Val::String("hello".into()))
            .await
            .expect("Failed to insert");
        let val = query(&trie, vid).await.expect("Failed to query");
        assert_eq!(Some(Val::String("hello".into())), val);
    }
}
