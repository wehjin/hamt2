use crate::_internal::KEY_VAL_TABLE;
use crate::_internal::Vid;
use crate::_internal::db_trie::subtrie_at;
use crate::{Ein, Val};
use crate::trie::*;
use crate::{QueryError, TransactError};

const VAL_TYPE_U32: u8 = 16;
const VAL_TYPE_STRING: u8 = 17;
const VAL_TYPE_U64: u8 = 18;
const VAL_TYPE_BYTES: u8 = 19;
const VAL_TYPE_EIN: u8 = 20;

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
        Val::Ein(e) => {
            let mut b = vec![VAL_TYPE_EIN];
            b.extend_from_slice(&(e.to_i32() as u32).to_be_bytes());
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
        VAL_TYPE_EIN => Val::Ein(Ein(
            u32::from_be_bytes(bytes[1..5].try_into().expect("parse ein from bytes")) as i32,
        )),
        _ => unreachable!(),
    }
}

const SEARCH_SIZE: usize = 4000;

/// Inserts `val` into the value table, returning the updated trie and the
/// hash-consed `Vid` for the value. The input trie is left unchanged.
pub fn insert(trie: &SkyTrie, val: Val) -> Result<(SkyTrie, Vid), TransactError> {
    let val_table = subtrie_at(trie, [KEY_VAL_TABLE]);
    let bytes = bytes_from_val(&val);
    let mut vid = Vid::for_search(universal_hash::hash(&bytes, 0) & TRIE_KEY_MASK);
    for _ in 0..SEARCH_SIZE {
        match val_table.query(vid.to_id()) {
            None => {
                let new_val_table = val_table.insert(vid.to_id(), TrieValue::Bytes(bytes));
                let new_trie = trie.insert_deep([KEY_VAL_TABLE], new_val_table);
                debug_assert_eq!(query(&new_trie, vid).expect("should find val"), Some(val));
                return Ok((new_trie, vid));
            }
            Some(TrieValue::Bytes(existing)) if &existing == &bytes => {
                return Ok((trie.clone(), vid));
            }
            Some(_) => vid = vid.next_search(),
        }
    }
    Err(TransactError::NoSpaceInValueTable)
}

pub fn query(trie: &SkyTrie, vid: Vid) -> Result<Option<Val>, QueryError> {
    let value = subtrie_at(trie, [KEY_VAL_TABLE]).query(vid.to_id());
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
    use crate::{Ein, Val, val};

    #[test]
    fn insert_and_query() {
        let mut trie = SkyTrie::new();
        let mut vids = Vec::new();
        let mut vals = Vec::new();
        for i in 0..100 {
            let i_val = val(i);
            vals.push(i_val.clone());
            let (next, vid) = insert(&trie, i_val).expect("Failed to insert");
            vids.push(vid);
            trie = next;
        }
        for (vid, val) in vids.into_iter().zip(vals) {
            let table_val = query(&trie, vid).expect("Failed to query");
            assert_eq!(Some(val), table_val);
        }
    }

    #[test]
    fn negative_numbers() {
        let trie = SkyTrie::new();
        let (trie, vid) = insert(&trie, val(-1)).expect("Failed to insert");
        let table_val = query(&trie, vid).expect("Failed to query");
        assert_eq!(Some(val(-1)), table_val);
    }

    #[test]
    fn same_value_inserted_twice() {
        let trie = SkyTrie::new();
        let (trie, vid) = insert(&trie, val(101)).expect("Failed to insert");
        let (trie, vid2) = insert(&trie, val(101)).expect("Failed to insert");
        assert_eq!(vid, vid2);
        let table_val = query(&trie, vid).expect("Failed to query");
        assert_eq!(Some(val(101)), table_val);
    }

    #[test]
    fn string_insert_and_query() {
        let trie = SkyTrie::new();
        let (trie, vid) = insert(&trie, Val::String("hello".into())).expect("Failed to insert");
        let val = query(&trie, vid).expect("Failed to query");
        assert_eq!(Some(Val::String("hello".into())), val);
    }

    #[test]
    fn u64_insert_and_query() {
        let trie = SkyTrie::new();
        let (trie, vid) = insert(&trie, Val::U64(u64::MAX)).expect("Failed to insert");
        let val = query(&trie, vid).expect("Failed to query");
        assert_eq!(Some(Val::U64(u64::MAX)), val);
    }

    #[test]
    fn bytes_insert_and_query() {
        let trie = SkyTrie::new();
        let (trie, vid) = insert(&trie, Val::Bytes(vec![1, 2, 3, 4])).expect("Failed to insert");
        let (trie, vid_empty) = insert(&trie, Val::Bytes(vec![])).expect("Failed to insert");
        let val = query(&trie, vid).expect("Failed to query");
        assert_eq!(Some(Val::Bytes(vec![1, 2, 3, 4])), val);
        let val_empty = query(&trie, vid_empty).expect("Failed to query");
        assert_eq!(Some(Val::Bytes(vec![])), val_empty);
    }

    #[test]
    fn distinct_types_get_distinct_vids() {
        let trie = SkyTrie::new();
        let (trie, u32_vid) = insert(&trie, Val::U32(5)).expect("Failed to insert");
        let (trie, u64_vid) = insert(&trie, Val::U64(5)).expect("Failed to insert");
        let (_trie, bytes_vid) = insert(&trie, Val::Bytes(vec![5])).expect("Failed to insert");
        assert_ne!(u32_vid, u64_vid);
        assert_ne!(u32_vid, bytes_vid);
        assert_ne!(u64_vid, bytes_vid);
    }

    #[test]
    fn ein_insert_and_query() {
        let trie = SkyTrie::new();
        let (trie, vid) = insert(&trie, Val::Ein(Ein(42))).expect("Failed to insert");
        let val = query(&trie, vid).expect("Failed to query");
        assert_eq!(Some(Val::Ein(Ein(42))), val);
    }

    #[test]
    fn ein_distinct_from_u32() {
        let trie = SkyTrie::new();
        let (trie, u32_vid) = insert(&trie, Val::U32(42)).expect("Failed to insert");
        let (_trie, ein_vid) = insert(&trie, Val::Ein(Ein(42))).expect("Failed to insert");
        assert_ne!(u32_vid, ein_vid);
    }
}
