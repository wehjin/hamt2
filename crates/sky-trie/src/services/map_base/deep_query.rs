use crate::services::map_base;
use crate::{DeepKey, MapBase, TrieValue};

pub fn query_value_deep<const N: usize>(root: MapBase, key: [i32; N]) -> Option<TrieValue> {
    let deep_key = DeepKey::from(key);
    let mut current_map_base = root.clone();
    let last_index = N - 1;
    for i in 0..=last_index {
        match map_base::query_value(current_map_base, deep_key[i].clone()) {
            None => {
                return None;
            }
            Some(value) => {
                if i < last_index {
                    let TrieValue::Trie(sky_trie) = value else {
                        // A non-trie value has no sub-trie below it.
                        return None;
                    };
                    current_map_base = sky_trie.root;
                } else {
                    return Some(value);
                }
            }
        }
    }
    unreachable!();
}
