use crate::db::{Db, Schema};
use crate::traits::DbQuery;
use crate::traits::Find;
use crate::trie::prelude::*;
use sky_types::storage::{BaseView, BaseEdit};

/// A read-only snapshot of a [`Db`], for running queries only.
#[derive(Debug, Clone)]
pub struct DbReader<S: BaseView + Clone + Send> {
    schema: Schema,
    read_trie: TrieReader<S>,
}

impl<S: BaseView + Clone + Send> DbReader<S> {
    /// Produces a read-only snapshot of the given `db`. The `db` remains fully
    /// usable afterward; writes made after this call are invisible to the
    /// reader.
    pub fn load<T>(db: &Db<T>) -> Self
    where
        T: BaseEdit<Snapshot = S>,
    {
        let schema = db.schema.clone();
        let read_trie = db.trie.snapshot();
        DbReader { schema, read_trie }
    }

    pub fn start(schema: Schema, storage: &impl BaseView<Snapshot = S>) -> Self {
        let read_trie: TrieReader<S> = TrieReader::<S>::new(storage.snapshot());
        DbReader { schema, read_trie }
    }
}

impl<S: BaseView + Clone + Send> DbQuery for DbReader<S> {
    fn find<F: Find>(&self, find: F) -> impl Future<Output = Vec<F::Output>> {
        find.apply(&self.read_trie, &self.schema)
    }
}
