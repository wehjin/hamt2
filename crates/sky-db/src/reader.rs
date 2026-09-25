use crate::db::{Db, Schema};
use crate::traits::DbQuery;
use crate::traits::Find;
use sky_types::storage::{BaseStore, TrieView};
use sky_types::trie::BaseView;

/// A read-only snapshot of a [`Db`], for running queries only.
#[derive(Debug, Clone)]
pub struct DbReader<S: BaseStore> {
    schema: Schema,
    read_trie: TrieView<S>,
}

impl<S: BaseStore + Send + Sync> DbReader<S> {
    /// Produces a read-only snapshot of the given `db`. The `db` remains fully
    /// usable afterward; writes made after this call are invisible to the
    /// reader.
    pub fn load(db: &Db<S>) -> Self {
        let schema = db.schema.clone();
        let read_trie = db.trie.snapshot();
        DbReader { schema, read_trie }
    }

    pub fn start(schema: Schema, read_trie: TrieView<S>) -> Self {
        DbReader { schema, read_trie }
    }
}

impl<S: BaseStore + Send + Sync> DbQuery for DbReader<S> {
    async fn find<F: Find>(&self, find: F) -> Vec<F::Output> {
        find.apply(&self.read_trie, &self.schema).await
    }
}
