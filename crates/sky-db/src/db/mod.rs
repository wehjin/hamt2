pub mod db_trie;
pub mod query;
pub mod transact;
pub mod types;

use crate::db::attr_spec::DbSpec;
use crate::db::attribute::Attribute;
use crate::db::types::MaxEid;
use crate::error::ConnectError;
use crate::reader::DbReader;
use crate::schema;
pub use crate::types::*;
use sky_types::storage::{BaseStore, ReadStorageError, StorageStatus, TrieLoad};
use sky_types::trie::{Base, BaseId, BaseRead};
pub use types::*;

#[derive(Debug)]
pub struct Db<S: BaseStore> {
    pub(crate) schema: Schema,
    pub(crate) trie: TrieLoad<S>,
}

/// Production methods for Db
impl<S: BaseStore + Send + Sync> Db<S> {
    pub fn to_reader(&self) -> DbReader<S> {
        DbReader::load(self)
    }
}

/// Construction methods for Db
impl<S: BaseStore + Send + Sync> Db<S> {
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn status(&self) -> StorageStatus {
        let max_id = self.trie.max_id();
        let root = self.trie.read_root();
        StorageStatus { max_id, root }
    }

    /// Keep until we figure out a better api for sky-server.
    pub async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        self.trie.read_base(id).await
    }

    pub async fn new(storage: S, db_spec: impl Into<DbSpec>) -> Result<Self, ConnectError> {
        let db_spec = db_spec.into();
        let attr_specs = db_spec.as_ref();
        let (schema, trie) = {
            let mut trie = TrieLoad::load(storage);
            let mut max_eid = MaxEid::read(&trie).await?;
            let mut schema = Schema::starter();
            {
                let eins = max_eid.take(attr_specs.len());
                let attributes = eins
                    .into_iter()
                    .zip(attr_specs)
                    .map(|(ein, spec)| Attribute::new(ein, spec.clone()));
                schema.extend(attributes);
            }
            trie.edit(async |trie| {
                schema::save(&schema, trie, Txid::SETUP).await?;
                db_trie::set_max_tx(trie, Txid::FLOOR).await?;
                max_eid.write(trie).await?;
                Ok(())
            })
            .await
            .map_err(|e| ConnectError::TrieEdit(e))?;
            (schema, trie)
        };
        let db = Db { schema, trie };
        Ok(db)
    }

    pub async fn load(storage: S) -> Self {
        let starter_db = Db {
            schema: Schema::starter(),
            trie: TrieLoad::load(storage),
        };
        let db = Db {
            schema: schema::load(&starter_db).await,
            trie: starter_db.trie,
        };
        db
    }
}

impl<S: BaseStore + Clone> Db<S> {
    pub fn close(self) -> S {
        self.trie.close()
    }
}
