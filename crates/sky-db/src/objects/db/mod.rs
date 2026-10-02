pub mod query;
pub mod transact;

use crate::_internal::MaxEid;
use crate::_internal::db_trie;
use crate::_internal::schema;
use crate::Schema;
use crate::attr_spec::DbSpec;
use crate::attribute::Attribute;
use crate::errors::ConnectError;
use crate::objects::reader::DbReader;
use crate::trie::{Base, Buffer, BufferIndex, TrieSnap};
use crate::trie_storage::{MemView, StorageStatus};
use crate::types::Txid;

#[derive(Debug)]
pub struct Db {
    pub(crate) schema: Schema,
    pub(crate) trie: MemView,
}

/// Production methods for Db
impl Db {
    pub fn to_reader(&self) -> DbReader<MemView> {
        let schema = self.schema.clone();
        let read_trie = self.trie.snapshot();
        DbReader::start(schema, read_trie)
    }
}

/// Construction methods for Db
impl Db {
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn status(&self) -> StorageStatus {
        let view = self.trie.snapshot();
        let max_id = view.max_index();
        let root = view.get_root();
        StorageStatus { max_id, root }
    }

    /// Keep until we figure out a better api for sky-server.
    pub async fn read_base(&self, id: BufferIndex, size: usize) -> Base {
        self.trie.snapshot().get_base(id, size).await
    }

    pub async fn new(storage: MemView, db_spec: impl Into<DbSpec>) -> Result<Self, ConnectError> {
        let db_spec = db_spec.into();
        let attr_specs = db_spec.as_ref();
        let (schema, trie) = {
            let mut trie = storage;
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

    pub async fn load(storage: MemView) -> Self {
        let starter_db = Db {
            schema: Schema::starter(),
            trie: storage,
        };
        let db = Db {
            schema: schema::load(&starter_db).await,
            trie: starter_db.trie,
        };
        db
    }
}

impl Db {
    pub fn close(self) -> MemView {
        self.trie
    }
}
