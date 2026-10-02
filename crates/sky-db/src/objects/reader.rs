use crate::Schema;
use crate::traits::DbQuery;
use crate::traits::Find;
use sky_trie::trie::StoreView;

/// A read-only snapshot of a [`Db`], for running queries only.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DbReader<S: StoreView> {
    schema: Schema,
    read_trie: S,
}

impl<S: StoreView> DbReader<S> {
    pub fn start(schema: Schema, read_trie: S) -> Self {
        DbReader { schema, read_trie }
    }
}

impl<S: StoreView> DbQuery for DbReader<S> {
    async fn find<F: Find>(&self, find: F) -> Vec<F::Output> {
        find.apply(&self.read_trie, &self.schema).await
    }
}
