use crate::trie::{DeepKey, MapBase, TrieRead, TrieQueryError, TrieValue, map_base};

pub async fn deep_query_value<const N: usize, S: TrieRead>(
    root: MapBase,
    key: [i32; N],
    storage: &S,
) -> Result<Option<TrieValue>, TrieQueryError> {
    let deep_key = DeepKey::from(key);
    let mut current_map_base = root.clone();
    let last_index = N - 1;
    for i in 0..=last_index {
        match map_base::query_value(current_map_base, deep_key[i].clone(), storage).await? {
            None => {
                return Ok(None);
            }
            Some(value) => {
                if i < last_index {
                    let TrieValue::SubTrie(map_base) = value else {
                        // A non-map value has no sub-trie below it.
                        return Ok(None);
                    };
                    current_map_base = map_base;
                } else {
                    return Ok(Some(value));
                }
            }
        }
    }
    unreachable!();
}
