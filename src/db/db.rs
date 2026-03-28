use crate::db::component::{val_table, MaxEid};
use crate::db::core::attr::Attr;
use crate::db::core::datom::Datom;
use crate::db::core::ent::Ent;
use crate::db::find::EntsWithAttr;
use crate::db::find::Rule;
use crate::db::find::ValsWithEntAttr;
use crate::db::key::{KEY_AEVT, KEY_EAVT, KEY_MAX_TXID};
use crate::db::txid::Txid;
use crate::db::{Eid, Val};
use crate::space::Space;
use crate::trie::mem::value::MemValue;
use crate::trie::space::trie::SpaceTrie;
use crate::{LoadError, QueryError, TransactError};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Schema {
    map: HashMap<Attr, Eid>,
}
impl Schema {
    fn basic() -> Self {
        Self {
            map: HashMap::from([(Attr::DB_IDENT, Eid::DB_IDENT)]),
        }
    }
    pub fn new(attrs: Vec<Attr>, eids: Vec<Eid>) -> Self {
        let mut schema = Self::basic();
        let more = attrs.iter().cloned().zip(eids).collect::<Vec<_>>();
        schema.extend(more);
        schema
    }
    pub fn save<T: Space>(
        &self,
        mut trie: SpaceTrie<T>,
        txid: Txid,
    ) -> Result<SpaceTrie<T>, TransactError> {
        for (at, a_ent) in self.map.iter() {
            let ident = Val::from(at.to_ident().as_str());
            trie = add(trie, &self.map, *a_ent, Attr::DB_IDENT, ident, &txid)?;
        }
        Ok(trie)
    }
    pub fn load<T: Space>(attrs: Vec<Attr>, trie: &SpaceTrie<T>) -> Result<Self, LoadError> {
        let attrs_by_ident: HashMap<String, Attr> = attrs
            .into_iter()
            .map(|attr| (attr.to_ident(), attr))
            .collect();
        // Read attr eids from the trie.
        let mut schema = Schema::basic();
        let ents_with_idents = {
            let mut rule = EntsWithAttr::new("e", Attr::DB_IDENT);
            rule.update(&trie, &schema)?;
            rule.results("e").to_vec()
        };
        for ent_with_ident in ents_with_idents {
            let mut rule = ValsWithEntAttr::new("v", ent_with_ident, Attr::DB_IDENT);
            rule.update(&trie, &schema)?;
            let ident = rule
                .results("v")
                .first()
                .expect("val should exist")
                .as_str();
            if let Some(attr) = attrs_by_ident.get(ident) {
                schema.insert(*attr, ent_with_ident.to_eid());
            }
        }
        // Confirm all requested attrs have eids.
        for attr in attrs_by_ident.values() {
            if !schema.contains_key(attr) {
                return Err(LoadError::UnknownAttr(*attr));
            }
        }
        Ok(schema)
    }
}
impl Deref for Schema {
    type Target = HashMap<Attr, Eid>;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}
impl DerefMut for Schema {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

#[derive(Debug)]
pub struct Db<T: Space> {
    schema: Schema,
    trie: SpaceTrie<T>,
    space: T,
}

impl<T: Space> Db<T> {
    pub fn close(self) -> T {
        self.space
    }

    pub fn new(mut space: T, attrs: Vec<Attr>) -> Result<Self, TransactError> {
        let schema = {
            let mut trie = SpaceTrie::connect(&space)?;
            let mut max_eid = MaxEid::read(&trie)?;
            let attr_eids = max_eid.take(attrs.len());
            let schema = Schema::new(attrs, attr_eids);
            trie = schema.save(trie, Txid::SETUP)?;
            trie = set_max_tx(trie, Txid::FLOOR)?;
            trie = max_eid.write(trie)?;
            trie.commit(&mut space)?;
            schema
        };
        let trie = SpaceTrie::connect(&space)?;
        let db = Self {
            schema,
            trie,
            space,
        };
        Ok(db)
    }

    pub fn load(space: T, attrs: Vec<Attr>) -> Result<Self, LoadError> {
        let trie = SpaceTrie::connect(&space)?;
        let schema = Schema::load(attrs, &trie)?;
        Ok(Self {
            schema,
            trie,
            space,
        })
    }
}

impl<T: Space> Db<T> {
    pub fn transact(self, datoms: Vec<Datom>) -> Result<Self, TransactError> {
        let mut new_eids = HashMap::new();
        let mut max_eid = MaxEid::read(&self.trie)?;
        match datoms.is_empty() {
            true => Ok(self),
            false => {
                let tx = self.max_tx()?;
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
                            add(trie, &attr_map, eid, a, v, &tx)?
                        }
                    }
                }
                trie = set_max_tx(trie, tx + 1)?;
                trie = max_eid.write(trie)?;
                trie.commit(&mut space)?;
                let db = Self {
                    schema: attr_map,
                    trie: SpaceTrie::connect(&space)?,
                    space,
                };
                Ok(db)
            }
        }
    }
}

impl<T: Space> Db<T> {
    pub fn max_tx(&self) -> Result<Txid, QueryError> {
        let Some(MemValue::U32(value)) = self.trie.query_value(KEY_MAX_TXID)? else {
            panic!("max_tx not found");
        };
        Ok(Txid::from(value))
    }

    pub fn find(&self, rule: &mut impl Rule) -> Result<bool, QueryError> {
        rule.update(&self.trie, &self.schema)
    }

    pub fn find_val(&self, e: Ent, a: Attr) -> Result<Option<Val>, QueryError> {
        let mut rule = ValsWithEntAttr::new("v", e, a);
        self.find(&mut rule)?;
        let vals = rule.results("v");
        match vals.first() {
            None => Ok(None),
            Some(v) => Ok(Some(v.clone())),
        }
    }
}

fn add<T: Space>(
    trie: SpaceTrie<T>,
    attr_map: &HashMap<Attr, Eid>,
    e: Eid,
    a: Attr,
    v: Val,
    t: &Txid,
) -> Result<SpaceTrie<T>, TransactError> {
    let eid = e.to_i32();
    let aid = attr_map.get(&a).expect("attr should exist").to_i32();
    let (mut trie, vid) = val_table::insert(trie, v)?;
    let tid = t.u32();
    let eavt_key = [KEY_EAVT, eid, aid, vid.to_id()];
    let aevt_key = [KEY_AEVT, aid, eid, vid.to_id()];
    trie = trie.deep_insert(eavt_key, MemValue::from(tid))?;
    trie = trie.deep_insert(aevt_key, MemValue::from(tid))?;
    Ok(trie)
}

fn set_max_tx<T: Space>(trie: SpaceTrie<T>, max_tx: Txid) -> Result<SpaceTrie<T>, TransactError> {
    trie.insert(KEY_MAX_TXID, MemValue::from(max_tx.u32()))
}
