use crate::db::component::MaxEid;
use crate::db::component::ent_eid::EntEid;
use crate::db::component::db_trie;
use crate::db::{val, Dat, Datom, Db, Ent};
use crate::space::Space;
use crate::trie::SpaceTrie;
use crate::TransactError;

impl<T: Space> Db<T> {
    pub async fn transact(self, datoms: impl Into<Vec<Datom>>) -> Result<Self, TransactError> {
        let datoms = datoms.into();
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
                let ent_eid = EntEid::new(&datoms, &mut max_eid);
                for datom in datoms {
                    trie = match datom {
                        Datom::Add(ent, attr, dat) => {
                            let eid = match ent {
                                Ent::Id(eid) => eid,
                                Ent::Temp(name) => ent_eid[name],
                            };
                            let val = match dat {
                                Dat::Val(val) => val,
                                Dat::Ent(ent) => {
                                    let eid = match ent {
                                        Ent::Id(eid) => eid,
                                        Ent::Temp(name) => ent_eid[name],
                                    };
                                    val(eid)
                                }
                            };
                            db_trie::add(trie, &attr_map, eid, attr, val, &tx).await?
                        }
                    }
                }
                trie = db_trie::set_max_tx(trie, tx + 1).await?;
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
