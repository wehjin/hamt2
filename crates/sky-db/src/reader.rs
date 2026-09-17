use crate::db::{Db, Schema};
use crate::find::Find;
use crate::query::DbQuery;
use crate::trie::prelude::*;

/// A read-only snapshot of a [`Db`], for running queries only.
#[derive(Debug)]
pub struct DbReader<S: ReadStorage> {
    schema: Schema,
    read_trie: TrieReader<S>,
}

impl<S: ReadStorage> DbReader<S> {
    /// Produces a read-only snapshot of the given `db`. The `db` remains fully
    /// usable afterward; writes made after this call are invisible to the
    /// reader.
    pub fn load<T>(db: &Db<T>) -> Self
    where
        T: ReadWriteStorage<Snapshot = S>,
    {
        let schema = db.schema.clone();
        let read_trie = TrieReader::connect(db.trie.storage().snapshot());
        DbReader { schema, read_trie }
    }
}

impl<S: ReadStorage> DbQuery for DbReader<S> {
    fn find<F: Find>(&self, find: F) -> impl Future<Output = Vec<F::Output>> {
        find.apply(&self.read_trie, &self.schema)
    }
}
