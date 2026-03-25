use crate::hamt::trie::core::map_base::TrieMapBase;
use crate::hamt::trie::core::query::QueryKeysValues;
use crate::hamt::trie::mem::value::MemValue;
use crate::hamt::trie::space::map_base::SpaceMapBase;
use crate::space::Read;
use crate::QueryError;

impl QueryKeysValues for TrieMapBase {
    fn query_keys_values(&self, reader: &impl Read) -> Result<Vec<(i32, MemValue)>, QueryError> {
        let mut out = Vec::new();
        match self {
            TrieMapBase::Mem(map, base) => {
                let slot_count = map.slot_count();
                debug_assert_eq!(slot_count, base.len());
                for base_index in 0..slot_count {
                    let keys_values = base[base_index].query_key_values(reader)?;
                    out.extend(keys_values);
                }
            }
            TrieMapBase::Space(slot_value) => {
                let map_base = SpaceMapBase::assert(*slot_value);
                out.extend(map_base.query_keys_values(reader)?);
            }
        }
        Ok(out)
    }
}
