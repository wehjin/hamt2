use crate::db::component::db_trie;
use crate::db::find::{DbFinder, EinAttrAny, Find};
use crate::db::query::DbQuery;
use crate::db::{Attr, Db, Ein, Schema, Val};
use crate::trie::ReadTrie;
use crate::trie::TrieQuery;
use crate::trie::base_storage::{BaseStorageRead, BaseStorageReadWrite};
use crate::{LoadError, QueryError};
use futures::FutureExt;

/// A read-only snapshot of a [`Db`], for running queries only.
#[derive(Debug)]
pub struct DbReader<S: BaseStorageRead> {
    schema: Schema,
    read_trie: ReadTrie<S>,
}

impl<S: BaseStorageRead> DbReader<S> {
    /// Produces a read-only snapshot of the given `db`. The `db` remains fully
    /// usable afterward; writes made after this call are invisible to the
    /// reader.
    pub async fn load<T>(db: &Db<T>) -> Result<Self, LoadError>
    where
        T: BaseStorageReadWrite<ReadOnly = S>,
    {
        let schema = db.schema.clone();
        let read_trie = ReadTrie::connect(db.trie.storage().to_readonly()).await?;
        Ok(DbReader { schema, read_trie })
    }

    // List the entities in the database.
    pub async fn list_entities(&self) -> Vec<Ein> {
        db_trie::list_entities(&self.read_trie).await
    }
}

impl<S: BaseStorageRead> DbQuery for DbReader<S> {
    fn find_val(
        &self,
        e: impl Into<Ein>,
        a: Attr,
    ) -> impl Future<Output = Result<Option<Val>, QueryError>> {
        let find = EinAttrAny::new(e, a);
        find.apply(&self.read_trie, &self.schema)
            .map(|result| result.map(|vals| vals.first().cloned()))
    }
}

impl<S: BaseStorageRead> DbFinder for DbReader<S> {
    fn find<F: Find>(&self, find: F) -> impl Future<Output = Result<Vec<F::Output>, QueryError>> {
        find.apply(&self.read_trie, &self.schema)
    }
}
