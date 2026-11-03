use crate::services::map_base;
use crate::{HashKey, MapBase, SkyTrie, TrieValue};

/// Inserts `value` at the deep path `key`, rebuilding the path from the leaf
/// back to the root. Unchanged subtries are shared, so this is a persistent
/// operation: the input `root` is left untouched.
///
/// An intermediate key that is absent (or holds a non-subtrie value) is treated
/// as an empty subtrie, so the path is created as needed.
pub fn insert_kv_deep<const N: usize>(root: MapBase, key: [i32; N], value: TrieValue) -> MapBase {
    fn go(mb: MapBase, key: &[i32], value: TrieValue) -> MapBase {
        let Some((&head, rest)) = key.split_first() else {
            return mb;
        };
        if rest.is_empty() {
            map_base::insert_kv(mb, HashKey::new(head), value)
        } else {
            let child = match map_base::query_value(mb.clone(), HashKey::new(head)) {
                Some(TrieValue::Trie(sky_trie)) => sky_trie.root,
                _ => MapBase::empty(),
            };
            let new_child = go(child, rest, value);
            map_base::insert_kv(mb, HashKey::new(head), TrieValue::Trie(SkyTrie { root: new_child }))
        }
    }
    go(root, &key, value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::map_base;

    #[test]
    fn insert_deep_and_query() {
        let root = MapBase::empty();
        let root = insert_kv_deep(root, [1, 2, 3], TrieValue::U32(42));
        assert_eq!(
            map_base::query_value_deep(root.clone(), [1, 2, 3]),
            Some(TrieValue::U32(42))
        );

        // Sibling paths coexist.
        let root = insert_kv_deep(root, [1, 2, 4], TrieValue::U32(43));
        assert_eq!(
            map_base::query_value_deep(root.clone(), [1, 2, 3]),
            Some(TrieValue::U32(42))
        );
        assert_eq!(
            map_base::query_value_deep(root.clone(), [1, 2, 4]),
            Some(TrieValue::U32(43))
        );
    }

    #[test]
    fn insert_deep_is_persistent() {
        let root = MapBase::empty();
        let next = insert_kv_deep(root.clone(), [1, 2, 3], TrieValue::U32(42));
        // The original root is unchanged.
        assert_eq!(map_base::query_value_deep(root, [1, 2, 3]), None);
        assert_eq!(
            map_base::query_value_deep(next, [1, 2, 3]),
            Some(TrieValue::U32(42))
        );
    }

    #[test]
    fn insert_deep_creates_missing_path() {
        let root = MapBase::empty();
        let root = insert_kv_deep(root, [5, 6, 7, 8], TrieValue::U32(99));
        assert_eq!(
            map_base::query_value_deep(root, [5, 6, 7, 8]),
            Some(TrieValue::U32(99))
        );
    }
}
