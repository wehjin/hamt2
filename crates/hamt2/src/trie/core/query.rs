use crate::trie::mem::value::MemValue;
use crate::{space, QueryError};

pub trait QueryKeysValues {
    fn query_keys_values(
        &self,
        reader: &impl space::Read,
    ) -> impl Future<Output = Result<Vec<(i32, MemValue)>, QueryError>>;
}
