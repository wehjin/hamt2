use crate::hamt::trie::core::query::QueryKeysValues;
use crate::hamt::trie::mem::value::MemValue;
use crate::hamt::trie::space::trie::SpaceTrie;
use crate::space::Space;
use crate::QueryError;

impl<T: Space> SpaceTrie<T> {
    pub fn query_keys_values(&self) -> Result<Vec<(i32, MemValue)>, QueryError> {
        self.map_base.query_keys_values(&self.reader)
    }
}
