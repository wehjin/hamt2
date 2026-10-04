use crate::_internal::schema;
use crate::_internal::{EntEid, db_trie, val_table};
use crate::_internal::{KEY_MAX_TXID, MaxEid};
use crate::attr_spec::DbSpec;
use crate::attribute::AttributeDetails;
use crate::errors::ConnectError;
use crate::trie::SkyTrie;
use crate::types::Txid;
use crate::{
    Attr, Attribute, Dat, Datom, Ein, Ent, Fill, QueryError, Schema, Transact, TransactError, Val, val,
};
use sky_trie::{Query, TrieValue};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Pod {
    pub(crate) schema: Schema,
    pub(crate) trie: SkyTrie,
}

/// Constructors
impl Pod {
    pub async fn new(db_spec: impl Into<DbSpec>) -> Result<Self, ConnectError> {
        let storage = SkyTrie::new();
        let db_spec = db_spec.into();
        let attr_specs = db_spec.as_ref();
        let (schema, trie) = {
            let mut trie = storage;
            let mut max_eid = MaxEid::read(&trie).await?;
            let mut schema = Schema::starter();
            {
                let attr_eins = max_eid.take(attr_specs.len());
                let attributes = attr_eins
                    .clone()
                    .into_iter()
                    .zip(attr_specs)
                    .map(|(ein, spec)| AttributeDetails::new(ein, spec.clone()))
                    .collect::<Vec<_>>();
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
        let db = Pod { schema, trie };
        Ok(db)
    }
}

/// Queries
impl Pod {
    pub fn schema(&self) -> &Schema {
        &self.schema
    }
    pub async fn max_tx(&self) -> Result<Txid, QueryError> {
        let Some(TrieValue::U32(value)) = self.trie.query(KEY_MAX_TXID) else {
            panic!("max_tx not found");
        };
        Ok(Txid::from(value))
    }

    pub fn ev_iter(&self, a: Attr) -> impl Iterator<Item = (i32, Val)> {
        db_trie::ev_iter(&self.trie, a, &self.schema)
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
            for (_attr_ein, vid) in db_trie::list_entity_fills(&self.trie, ein, attr_ein) {
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
        let mut max_eid = MaxEid::read(&self.trie).await?;
        let tx = self.max_tx().await?;
        self.trie
            .edit(async |trie| {
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
                    db_trie::with_update(trie, &self.schema, eid, attr, val, dir, &tx).await?;
                }
                db_trie::set_max_tx(trie, tx + 1).await?;
                max_eid.write(trie).await?;
                Ok(())
            })
            .await
            .map_err(|e| TransactError::Rewind(e))?;
        Ok(self)
    }
}
