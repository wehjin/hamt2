use crate::db::attr_spec::DbSpec;
use crate::db::component::MaxEid;
use crate::db::component::db_trie;
use crate::db::{Attr, Db, Txid};
use crate::trie::prelude::*;
use crate::types::schema::Schema;
use crate::types::schema::attribute::Attribute;
use crate::{LoadError, TransactError};

impl<S: ReadWriteTrieStorage> Db<S> {
    pub async fn new(storage: S, db_spec: impl Into<DbSpec>) -> Result<Self, TransactError> {
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
