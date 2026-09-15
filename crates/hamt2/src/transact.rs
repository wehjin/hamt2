use sky_types::db::TransactError;
use crate::db::Db;
use crate::db::db_trie;
use crate::db::types::MaxEid;
use crate::db::types::ent_eid::EntEid;
use crate::trie::prelude::*;
use sky_types::db::{Dat, Datom, Ent, val};

impl<S: ReadWriteTrieStorage> Db<S> {
    pub async fn transact(self, datoms: impl Into<Vec<Datom>>) -> Result<Self, TransactError> {
        let datoms = datoms.into();
        let mut max_eid = MaxEid::read(&self.trie).await?;
        match datoms.is_empty() {
            true => Ok(self),
            false => {
                let tx = self.max_tx().await?;

                let Self {
                    schema: attr_map,
                    mut trie,
                } = self;
                let ent_eid = EntEid::new(&datoms, &mut max_eid);
                for datom in datoms {
                    let eid = match datom.ent {
                        Ent::Id(eid) => eid,
                        Ent::Temp(name) => ent_eid[name],
                    };
                    let attr = datom.attr;
                    let val = match datom.dat {
                        Dat::Val(val) => val,
                        Dat::Ent(ent) => {
                            let eid = match ent {
                                Ent::Id(eid) => eid,
                                Ent::Temp(name) => ent_eid[name],
                            };
                            val(eid)
                        }
                    };
                    let dir = datom.dir;
                    trie = db_trie::with_update(trie, &attr_map, eid, attr, val, dir, &tx).await?;
                }
                trie = db_trie::set_max_tx(trie, tx + 1).await?;
                trie = max_eid.write(trie).await?;
                trie = trie.commit().await?;
                let db = Self {
                    schema: attr_map,
                    trie,
                };
                Ok(db)
            }
        }
    }
}
