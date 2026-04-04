use crate::db::{Attr, Db, Txid};
use crate::db::component::MaxEid;
use crate::db::schema::Schema;
use crate::space::Space;
use crate::{LoadError, TransactError};
use crate::db::component::trie;
use crate::trie::SpaceTrie;

impl<T: Space> Db<T> {
    pub async fn new(mut space: T, attrs: Vec<Attr>) -> Result<Self, TransactError> {
        let schema = {
            let mut trie = SpaceTrie::connect(&space).await?;
            let mut max_eid = MaxEid::read(&trie).await?;
            let attr_eids = max_eid.take(attrs.len());
            let schema = Schema::new(attrs, attr_eids);
            trie = schema.save(trie, Txid::SETUP).await?;
            trie = trie::trie_set_max_tx(trie, Txid::FLOOR).await?;
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

    pub async fn load(space: T, attrs: Vec<Attr>) -> Result<Self, LoadError> {
        let trie = SpaceTrie::connect(&space).await?;
        let schema = Schema::load(attrs, &trie).await?;
        Ok(Self {
            schema,
            trie,
            space,
        })
    }

    pub fn close(self) -> T {
        self.space
    }
}