use crate::db::component::u31::{U31Builder, U31Streamer};
use crate::db::key::KEY_VAL_TABLE;
use crate::db::Val;
use crate::db::Vid;
use crate::hamt::trie::mem::value::MemValue;
use crate::hamt::trie::space::SpaceTrie;
use crate::space::Space;
use crate::{hash, QueryError, TransactError};
use redb::Value;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::space::mem::MemSpace;

    #[test]
    fn insert_and_query() {
        let space = MemSpace::new();
        let mut trie = SpaceTrie::connect(&space).expect("Failed to connect to MemSpace");
        let mut vids = Vec::new();
        let mut vals = Vec::new();
        for i in 0..100 {
            let val = Val::U32(i);
            vals.push(val.clone());
            let (new_trie, vid) = insert(trie, val).expect("Failed to insert");
            trie = new_trie;
            vids.push(vid);
        }
        for (vid, val) in vids.into_iter().zip(vals) {
            let table_val = query(&trie, vid).expect("Failed to query");
            assert_eq!(Some(val), table_val);
        }
    }

    #[test]
    fn string_insert_and_query() {
        let space = MemSpace::new();
        let trie = SpaceTrie::connect(&space).expect("Failed to connect to MemSpace");
        let (trie, vid) = insert(trie, Val::String("hello".into())).expect("Failed to insert");
        let val = query(&trie, vid).expect("Failed to query");
        assert_eq!(Some(Val::String("hello".into())), val);
    }
}

pub fn insert<T: Space>(
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
        let hash_trie = find_hash_trie(&trie, hash)?;
        match hash_trie {
            None => {
                let trie = insert_bytes(trie, hash, bytes, bytes_type)?;
                return Ok((trie, Vid::from_id(hash)));
            }
            Some(bytes_trie) => {
                if is_equal_bytes(&bytes_trie, bytes, bytes_type)? {
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

pub fn query<T: Space>(trie: &SpaceTrie<T>, vid: Vid) -> Result<Option<Val>, QueryError> {
    match find_hash_trie(trie, vid.to_id())? {
        None => Ok(None),
        Some(val_trie) => {
            let Some(MemValue::U32(bytes_len)) = val_trie.query_value(KEY_LEN)? else {
                panic!("Unexpected MemValue variant")
            };
            let Some(MemValue::U32(val_type)) = val_trie.query_value(KEY_VAL_TYPE)? else {
                panic!("Unexpected MemValue variant")
            };
            let builder = U31Builder::new(&val_trie, bytes_len as usize);
            let bytes = builder.collect::<Vec<_>>();
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
const KEY_LEN: i32 = -1;
const KEY_VAL_TYPE: i32 = -2;
const VAL_TYPE_U32: u8 = 0;

const VAL_TYPE_STRING: u8 = 1;

fn insert_bytes<T: Space>(
    mut trie: SpaceTrie<T>,
    hash: i32,
    bytes: &[u8],
    bytes_type: u8,
) -> Result<SpaceTrie<T>, TransactError> {
    let stream = U31Streamer::new(bytes);
    for (key, value) in stream {
        trie = trie.deep_insert([KEY_VAL_TABLE, hash, key], MemValue::U32(value))?;
    }
    trie = trie.deep_insert(
        [KEY_VAL_TABLE, hash, KEY_LEN],
        MemValue::U32(bytes.len() as u32),
    )?;
    trie = trie.deep_insert(
        [KEY_VAL_TABLE, hash, KEY_VAL_TYPE],
        MemValue::U32(bytes_type as u32),
    )?;
    Ok(trie)
}

fn is_equal_bytes<T: Space>(
    hash_trie: &SpaceTrie<T>,
    bytes: &[u8],
    bytes_type: u8,
) -> Result<bool, QueryError> {
    let Some(MemValue::U32(len)) = hash_trie.query_value(KEY_LEN)? else {
        panic!("Unexpected MemValue variant")
    };
    if len as usize != bytes.len() {
        return Ok(false);
    }
    let Some(MemValue::U32(val_type)) = hash_trie.query_value(KEY_VAL_TYPE)? else {
        panic!("Unexpected MemValue variant")
    };
    if val_type as u8 != bytes_type {
        return Ok(false);
    }
    let mut stream = U31Streamer::new(bytes);
    loop {
        if let Some((key, value)) = stream.next() {
            match hash_trie.query_value(key)? {
                None => return Ok(false),
                Some(mem_value) => {
                    let MemValue::U32(val) = mem_value else {
                        panic!("Unexpected MemValue variant")
                    };
                    if value != val {
                        return Ok(false);
                    }
                }
            }
            return Ok(true);
        } else {
            return Ok(true);
        }
    }
}

fn find_hash_trie<T: Space>(
    trie: &SpaceTrie<T>,
    hash: i32,
) -> Result<Option<SpaceTrie<T>>, QueryError> {
    let key = [KEY_VAL_TABLE, hash];
    match trie.deep_query_value(key)? {
        None => Ok(None),
        Some(mem_value) => {
            let bytes_trie = trie.to_subtrie_from_value(mem_value)?;
            Ok(Some(bytes_trie))
        }
    }
}
