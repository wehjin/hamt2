use crate::hamt::trie::core::query::QueryKeysValues;
use crate::hamt::trie::mem::value::MemValue;
use crate::hamt::trie::space::map_base::SpaceMapBase;
use crate::space::Read;
use crate::QueryError;

impl QueryKeysValues for SpaceMapBase {
    fn query_key_values(&self, reader: &impl Read) -> Result<Vec<(i32, MemValue)>, QueryError> {



        Ok(vec![])
    }
}
