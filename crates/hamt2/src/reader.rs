use crate::LoadError;
use crate::db::{Db, Schema};
use crate::find::Find;
use crate::query::DbQuery;
use crate::trie::prelude::*;

/// A read-only snapshot of a [`Db`], for running queries only.
#[derive(Debug)]
pub struct DbReader<S: ReadTrieStorage> {
    schema: Schema,
    read_trie: TrieReader<S>,
}

impl<S: ReadTrieStorage> DbReader<S> {
    /// Produces a read-only snapshot of the given `db`. The `db` remains fully
    /// usable afterward; writes made after this call are invisible to the
    /// reader.
    pub async fn load<T>(db: &Db<T>) -> Result<Self, LoadError>
    where
        T: ReadWriteTrieStorage<ReadOnly = S>,
    {
        let schema = db.schema.clone();
        let read_trie = TrieReader::connect(db.trie.storage().to_readonly()).await?;
        Ok(DbReader { schema, read_trie })
    }
}

impl<S: ReadTrieStorage> DbQuery for DbReader<S> {
    fn find<F: Find>(&self, find: F) -> impl Future<Output = Vec<F::Output>> {
        find.apply(&self.read_trie, &self.schema)
    }
}
