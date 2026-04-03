use std::collections::HashMap;
use crate::db::{trie, Datom, Db, Ent};
use crate::db::component::MaxEid;
use crate::space::Space;
use crate::TransactError;
use crate::trie::SpaceTrie;

impl<T: Space> Db<T> {
    pub async fn transact(self, datoms: impl Into<Vec<Datom>>) -> Result<Self, TransactError> {
        let datoms = datoms.into();
        let mut new_eids = HashMap::new();
        let mut max_eid = MaxEid::read(&self.trie).await?;
        match datoms.is_empty() {
            true => Ok(self),
            false => {
                let tx = self.max_tx().await?;
                let Self {
                    schema: attr_map,
                    mut space,
                    mut trie,
                } = self;
                for datom in datoms {
                    trie = match datom {
                        Datom::Add(e, a, v) => {
                            let eid = match e {
                                Ent::Id(eid) => eid,
                                Ent::Temp(name) => {
                                    if let Some(eid) = new_eids.get(name) {
                                        *eid
                                    } else {
                                        let eid =
                                            max_eid.take(1).pop().expect("max_eid should exist");
                                        new_eids.insert(name, eid);
                                        eid
                                    }
                                }
                            };
                            trie::trie_add(trie, &attr_map, eid, a, v, &tx).await?
                        }
                    }
                }
                trie = trie::trie_set_max_tx(trie, tx + 1).await?;
                trie = max_eid.write(trie).await?;
                trie.commit(&mut space).await?;
                let db = Self {
                    schema: attr_map,
                    trie: SpaceTrie::connect(&space).await?,
                    space,
                };
                Ok(db)
            }
        }
    }
}