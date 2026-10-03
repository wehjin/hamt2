use crate::Schema;
use crate::traits::DbQuery;
use crate::traits::Find;
use crate::trie::Map;

/// A read-only snapshot of a [`Db`], for running queries only.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DbReader<S: Map> {
    schema: Schema,
    read_trie: S,
}

impl<S: Map> DbReader<S> {
    pub fn start(schema: Schema, read_trie: S) -> Self {
        DbReader { schema, read_trie }
    }
}

impl<S: Map> DbQuery for DbReader<S> {
    async fn find<F: Find>(&self, find: F) -> Vec<F::Output> {
        find.apply(&self.read_trie, &self.schema).await
    }
}
