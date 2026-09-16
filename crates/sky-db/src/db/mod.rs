pub mod db_trie;
pub mod types;

use crate::LoadError;
use crate::db::attr_spec::DbSpec;
use crate::db::attribute::Attribute;
use crate::db::types::MaxEid;
use crate::error::ConnectError;
use crate::reader::DbReader;
pub use crate::types::*;
use sky_trie::Trie;
use sky_trie::prelude::ReadWriteStorage;
use sky_types::db::Attr;
pub use types::*;

#[derive(Debug)]
pub struct Db<S: ReadWriteStorage> {
    pub(crate) schema: Schema,
    pub(crate) trie: Trie<S>,
}

/// Production methods for Db
impl<S: ReadWriteStorage> Db<S> {
    pub async fn to_reader(&self) -> DbReader<S::Snapshot> {
        DbReader::load(self).await.expect("load reader")
    }
}

/// Construction methods for Db
impl<S: ReadWriteStorage> Db<S> {
    pub async fn new(storage: S, db_spec: impl Into<DbSpec>) -> Result<Self, ConnectError> {
        let db_spec = db_spec.into();
        let attr_specs = db_spec.as_ref();
        let (schema, trie) = {
            let mut trie = Trie::connect(storage).await?;
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
            trie = schema.save(trie, Txid::SETUP).await?;
            trie = db_trie::set_max_tx(trie, Txid::FLOOR).await?;
            trie = max_eid.write(trie).await?;
            trie = trie.commit().await?;
            (schema, trie)
        };
        let db = Db { schema, trie };
        Ok(db)
    }

    pub async fn load(storage: S, attrs: impl AsRef<[Attr]>) -> Result<Self, LoadError> {
        let attrs = attrs.as_ref();
        let starter_db = Db {
            schema: Schema::starter(),
            trie: Trie::connect(storage).await?,
        };
        let db = Db {
            schema: Schema::load(attrs, &starter_db).await?,
            trie: starter_db.trie,
        };
        Ok(db)
    }

    pub fn close(self) -> S {
        self.trie.close()
    }
}

pub fn query() -> Attr {
    Attr::from("db/query")
}
pub fn ident() -> Attr {
    Attr::from("db/ident")
}
pub fn cardinality() -> Attr {
    Attr::from("db/cardinality")
}
