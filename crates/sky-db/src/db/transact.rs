use crate::db::Db;
use crate::db::db_trie;
use crate::db::types::MaxEid;
use crate::db::types::ent_eid::EntEid;
use sky_types::db::Datom;
use sky_types::db::Transact;
use sky_types::db::TransactError;
use sky_types::db::{Dat, Ent, val};
use sky_types::storage::BaseStore;

impl<S: BaseStore + Send + Sync> Transact for Db<S> {
    async fn transact(self, datoms: impl Into<Vec<Datom>>) -> Result<Self, TransactError> {
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
                trie.edit(async |trie| {
                    let ent_eid = EntEid::new(&datoms, &mut max_eid);
                    for datom in datoms {
                        let eid = match &datom.ent {
                            Ent::Id(eid) => *eid,
                            Ent::Temp(name) => ent_eid[name.as_str()],
                        };
                        let attr = datom.attr;
                        let val = match datom.dat {
                            Dat::Val(val) => val,
                            Dat::Ent(ent) => {
                                let eid = match ent {
                                    Ent::Id(eid) => eid,
                                    Ent::Temp(name) => ent_eid[name.as_str()],
                                };
                                val(eid)
                            }
                        };
                        let dir = datom.dir;
                        db_trie::with_update(trie, &attr_map, eid, attr, val, dir, &tx).await?;
                    }
                    db_trie::set_max_tx(trie, tx + 1).await?;
                    max_eid.write(trie).await?;
                    Ok(())
                })
                .await
                .map_err(|e| TransactError::TrieEdit(e))?;
                let db = Self {
                    schema: attr_map,
                    trie,
                };
                Ok(db)
            }
        }
    }
}
