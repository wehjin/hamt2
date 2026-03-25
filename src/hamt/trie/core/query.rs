use crate::hamt::trie::mem::value::MemValue;
use crate::{space, QueryError};

pub trait QueryKeysValues {
    fn query_keys_values(
        &self,
        reader: &impl space::Read,
    ) -> Result<Vec<(i32, MemValue)>, QueryError>;
}
