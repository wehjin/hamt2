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
use sky_trie::Trie;
use sky_types::storage::{ReadStorage, ReadStorageError, ReadWriteStorage, StorageStatus};
use sky_types::trie::{SlotBase, SlotBaseId, TrieRead};
pub use types::*;

#[derive(Debug)]
pub struct Db<S: ReadWriteStorage> {
    pub(crate) schema: Schema,
    pub(crate) trie: Trie<S>,
}

/// Production methods for Db
impl<S: ReadWriteStorage> Db<S> {
    pub fn to_reader(&self) -> DbReader<S::Snapshot> {
        DbReader::load(self)
    }
}

/// Construction methods for Db
impl<S: ReadWriteStorage> Db<S> {
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn status(&self) -> StorageStatus {
        self.trie.status()
    }

    /// Keep until we figure out a better api for sky-server.
    pub async fn read_base(&self, id: SlotBaseId) -> Result<SlotBase, ReadStorageError> {
        self.trie.read_base(id).await
    }

    pub async fn new(storage: S, db_spec: impl Into<DbSpec>) -> Result<Self, ConnectError> {
        let db_spec = db_spec.into();
        let attr_specs = db_spec.as_ref();
        let (schema, trie) = {
            let mut trie = Trie::connect(storage);
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
            trie = schema::save(&schema, trie, Txid::SETUP).await?;
            trie = db_trie::set_max_tx(trie, Txid::FLOOR).await?;
            trie = max_eid.write(trie).await?;
            trie = trie.commit().await?;
            (schema, trie)
        };
        let db = Db { schema, trie };
        Ok(db)
    }

    pub async fn load(storage: S) -> Self {
        let starter_db = Db {
            schema: Schema::starter(),
            trie: Trie::connect(storage),
        };
        let db = Db {
            schema: schema::load(&starter_db).await,
            trie: starter_db.trie,
        };
        db
    }

    pub fn close(self) -> S {
        self.trie.close()
    }
}
