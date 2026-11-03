use crate::SkyTrie;
use crate::{Insert, Query, Snap, TrieValue};
use fixtures::{edit_assert_trie, error_edit_assert_trie, start_assert_trie};

#[test]
fn view_has_query() {
    let mem = SkyTrie::new().snapshot();
    let values = mem.query_all();
    assert_eq!(values, vec![]);
}

#[test]
fn view_has_snap() {
    let mem = SkyTrie::new().snapshot();
    let values = mem.snapshot().query_all();
    assert_eq!(values, vec![]);
}

#[test]
fn edit_inserts() {
    let insert_value = TrieValue::U32(34);
    let mut mem = SkyTrie::new();
    mem.edit(|edit| {
        edit.insert(33, insert_value.clone());
        Ok(())
    })
    .unwrap();
    let query_value = mem.query(33).unwrap();
    assert_eq!(query_value, insert_value)
}

#[test]
fn main_accumulates_ok_edit() {
    let mut trie = start_assert_trie();
    edit_assert_trie(&mut trie, 62);
    let values = trie.query_all();
    assert_eq!(values, vec![(62, TrieValue::U32(63))]);
}

#[test]
fn main_rejects_err_edit() {
    let mut trie = start_assert_trie();
    error_edit_assert_trie(&mut trie, 62);
    let values = trie.query_all();
    assert_eq!(values.len(), 0);
}

#[test]
fn main_accumulates_second_ok_edit() {
    let mut trie = start_assert_trie();
    edit_assert_trie(&mut trie, 20);
    edit_assert_trie(&mut trie, 30);
    let mut values = trie.query_all();
    values.sort_by(|(a, _), (b, _)| a.cmp(b));
    assert_eq!(
        values,
        vec![(20, TrieValue::U32(21)), (30, TrieValue::U32(31)),]
    );
}

#[test]
fn main_rejects_second_err_edit() {
    let mut trie = start_assert_trie();
    edit_assert_trie(&mut trie, 20);
    error_edit_assert_trie(&mut trie, 30);
    let mut values = trie.query_all();
    values.sort_by(|(a, _), (b, _)| a.cmp(b));
    assert_eq!(values, vec![(20, TrieValue::U32(21))]);
}

mod fixtures {
    use crate::SkyTrie;
    use crate::{Insert, Query, TrieValue};
    use anyhow::anyhow;

    pub fn start_assert_trie() -> SkyTrie {
        let trie = SkyTrie::new();
        let values = trie.query_all();
        assert_eq!(values.len(), 0);
        trie
    }

    pub fn edit_assert_trie(trie: &mut SkyTrie, tag: i32) {
        let out = trie
            .edit(|edit| {
                let value = TrieValue::U32(tag as u32 + 1);
                edit.insert(tag, value.clone());
                let query = edit.query(tag);
                assert_eq!(query, Some(value));
                Ok(tag + 2)
            })
            .unwrap();
        assert_eq!(out, tag + 2);
    }

    pub fn error_edit_assert_trie(trie: &mut SkyTrie, tag: i32) {
        let out = trie.edit::<_, ()>(|edit| {
            let value = TrieValue::U32(tag as u32 + 1);
            edit.insert(tag, value.clone());
            let query = edit.query(tag);
            assert_eq!(query, Some(value));
            Err(anyhow!("trouble in edit"))
        });
        assert!(out.is_err());
    }
}
