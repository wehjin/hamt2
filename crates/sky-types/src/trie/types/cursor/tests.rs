use crate::trie::{CursorPos, MapBase, key};

#[test]
fn cursor_has_active_root() {
    let cursor = CursorPos::default();
    let _ = cursor.active_root();
}

#[test]
fn cursor_ascends_and_descends() {
    let mut cursor = CursorPos::default();
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.ascend(), None);

    cursor.descend(33, MapBase::default());
    assert_eq!(cursor.depth(), 1);
    cursor.descend(34, MapBase::default());
    assert_eq!(cursor.depth(), 2);

    let mut keys = vec![];
    while let Some((key, _map_base)) = cursor.ascend() {
        keys.push(key);
    }
    assert_eq!(keys, vec![key(34), key(33)]);
}
