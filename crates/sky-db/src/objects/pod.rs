use crate::_internal::{EntEid, db_trie, rand_nonce, val_table};
use crate::attr_spec::DbSpec;
use crate::errors::ConnectError;
use crate::trie::SkyTrie;
use crate::{
    Attr, Attribute, Dat, Datom, Ein, Ent, Fill, Hash, Schema, Transact, TransactError, Tx, Val,
    datom, db, tx, val,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Pod {
    pub(crate) schema: Schema,
    pub(crate) trie: SkyTrie,
    pub(crate) order: HashMap<Ein, u32>,
    pub(crate) tip: Option<Hash>,
}

/// Constructors
impl Pod {
    pub async fn new(db_spec: impl Into<DbSpec>) -> Result<Self, ConnectError> {
        let db_spec = db_spec.into();
        let attr_specs = db_spec.as_ref();
        let (schema, trie, order, tip) = {
            let mut trie = SkyTrie::new();
            let mut schema = Schema::starter();
            let nonce = rand_nonce();
            let tx_ein = Ein::tx(&nonce);
            // Build the genesis datoms: schema declarations + tx metadata.
            let mut genesis = Vec::new();
            for spec in attr_specs {
                let ein = Ein::attr(&spec.attr);
                genesis.push(datom::add(ein, db::ident(), spec.attr.as_ident()));
                let card: Val = spec.cardinality.into();
                genesis.push(datom::add(ein, db::cardinality(), card));
            }
            genesis.push(datom::add(tx_ein, tx::nonce(), Val::Bytes(nonce.to_vec())));
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time before epoch")
                .as_secs();
            genesis.push(datom::add(tx_ein, tx::instant(), Val::U64(now)));
            let genesis_tx = Tx {
                prev: None,
                author: [0u8; 32],
                nonce,
                datoms: genesis,
            };
            let hash = genesis_tx.hash();
            trie.edit(async |trie| {
                for datom in &genesis_tx.datoms {
                    let eid = datom.ent.to_eid();
                    let attr = datom.attr.clone();
                    let val = match &datom.dat {
                        Dat::Val(val) => val.clone(),
                        Dat::Ent(_) => unreachable!("genesis datoms have no entity refs"),
                    };
                    db_trie::with_update(trie, &schema, eid, attr, val, datom.dir, tx_ein).await?;
                }
                Ok(())
            })
            .await
            .map_err(|e| ConnectError::TrieEdit(e))?;
            // Derive the schema from the genesis datoms.
            schema.apply(&genesis_tx.datoms);
            let mut order = HashMap::new();
            order.insert(tx_ein, 0);
            (schema, trie, order, Some(hash))
        };
        let pod = Pod {
            schema,
            trie,
            order,
            tip,
        };
        Ok(pod)
    }
}

/// Queries
impl Pod {
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn ev_iter(&self, a: Attr) -> impl Iterator<Item = (i32, Val)> {
        db_trie::ev_iter(&self.trie, a, &self.schema, &self.order)
    }
}

/// Find helpers
impl Pod {
    pub(crate) fn list_entities(&self) -> Vec<Ein> {
        db_trie::list_entities(&self.trie)
    }

    pub(crate) fn list_attr_eins(&self, ein: Ein) -> Vec<Ein> {
        db_trie::list_entity_attributes(&self.trie, ein)
            .into_iter()
            .map(|attr_ein| attr_ein.ein())
            .collect::<Vec<_>>()
    }

    pub(crate) fn list_fills(&self, ein: Ein) -> Vec<Fill> {
        let mut fills = Vec::new();
        for attr_ein in db_trie::list_entity_attributes(&self.trie, ein) {
            let attr = self
                .schema
                .find_attr(attr_ein.ein())
                .cloned()
                .expect("attr should exist for attr-ein");
            let cardinality = self.schema[attr.clone()].cardinality();
            for (_attr_ein, vid) in db_trie::list_entity_fills(
                &self.trie,
                ein,
                attr_ein,
                cardinality,
                &self.order,
            ) {
                let val = val_table::query(&self.trie, vid)
                    .expect("table should find val")
                    .expect("val should exist");
                fills.push(Fill(attr.clone(), val));
            }
        }
        fills
    }
}

/// Attributes
impl Pod {
    pub fn get_attribute(&self, attr: impl Into<Attr>) -> Option<Attribute> {
        let details = self.schema.get_details(attr);
        details.map(|details| Attribute::new(details, self.clone()))
    }
    pub async fn list_attributes(&self) -> Vec<Attribute> {
        self.schema
            .list_details()
            .into_iter()
            .map(|details| Attribute::new(details, self.clone()))
            .collect::<Vec<_>>()
    }
}

/// Transact
impl Transact for Pod {
    async fn transact(
        &mut self,
        datoms: impl Into<Vec<Datom>>,
    ) -> Result<&mut Self, TransactError> {
        let datoms = datoms.into();
        if datoms.is_empty() {
            return Ok(self);
        }
        let nonce = rand_nonce();
        let tx_ein = Ein::tx(&nonce);
        // Resolve temp entities and build the full datom set.
        let ent_eid = EntEid::new(&datoms);
        let mut all = Vec::new();
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
            all.push(Datom {
                ent: Ent::Id(eid),
                attr,
                dat: Dat::Val(val),
                dir: datom.dir,
            });
        }
        all.push(datom::add(tx_ein, tx::nonce(), Val::Bytes(nonce.to_vec())));
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time before epoch")
            .as_secs();
        all.push(datom::add(tx_ein, tx::instant(), Val::U64(now)));
        let tx = Tx {
            prev: self.tip,
            author: [0u8; 32],
            nonce,
            datoms: all,
        };
        let hash = tx.hash();
        self.trie
            .edit(async |trie| {
                for datom in &tx.datoms {
                    let eid = datom.ent.to_eid();
                    let attr = datom.attr.clone();
                    let val = match &datom.dat {
                        Dat::Val(val) => val.clone(),
                        Dat::Ent(_) => unreachable!("resolved datoms have no entity refs"),
                    };
                    db_trie::with_update(trie, &self.schema, eid, attr, val, datom.dir, tx_ein)
                        .await?;
                }
                Ok(())
            })
            .await
            .map_err(|e| TransactError::Rewind(e))?;
        let pos = self.order.len() as u32;
        self.order.insert(tx_ein, pos);
        self.tip = Some(hash);
        Ok(self)
    }
}
