use crate::db::attr_spec::DbSpec;
use crate::db::component::db_trie;
use crate::db::component::MaxEid;
use crate::db::schema::attribute::Attribute;
use crate::db::schema::Schema;
use crate::db::{Attr, Db, Txid};
use crate::space::Space;
use crate::trie::SpaceTrie;
use crate::{LoadError, TransactError};

impl<T: Space> Db<T> {
    pub async fn new(mut space: T, db_spec: impl Into<DbSpec>) -> Result<Self, TransactError> {
        let db_spec = db_spec.into();
        let attr_specs = db_spec.as_ref();
        let schema = {
            let mut trie = SpaceTrie::connect(&space).await?;
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
            trie.commit(&mut space).await?;
            schema
        };
        let trie = SpaceTrie::connect(&space).await?;
        let db = Self {
            schema,
            trie,
            space,
        };
        Ok(db)
    }

    pub async fn load(space: T, attrs: impl AsRef<[Attr]>) -> Result<Self, LoadError> {
        let attrs = attrs.as_ref();
        let starter_db = Self {
            schema: Schema::starter(),
            trie: SpaceTrie::connect(&space).await?,
            space,
        };
        Ok(Self {
            schema: Schema::load(attrs, &starter_db).await?,
            trie: starter_db.trie,
            space: starter_db.space,
        })
    }

    pub fn close(self) -> T {
        self.space
    }
}
