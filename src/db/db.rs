use crate::db::component::{val_table, MaxEid};
use crate::db::datom::{Attr, Datom, Ent, Val};
use crate::db::find::EntsWithAttr;
use crate::db::find::Rule;
use crate::db::find::ValsWithEntAttr;
use crate::db::key::{KEY_AEVT, KEY_EAVT, KEY_MAX_TXID};
use crate::db::txid::Txid;
use crate::hamt::trie::mem::value::MemValue;
use crate::hamt::trie::space::trie::SpaceTrie;
use crate::space::Space;
use crate::{LoadError, QueryError, TransactError};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Db<T: Space> {
    attr_map: HashMap<Attr, Ent>,
    trie: SpaceTrie<T>,
    space: T,
}

impl<T: Space> Db<T> {
    pub fn new(mut space: T, attrs: Vec<Attr>) -> Result<Self, TransactError> {
        let mut trie = SpaceTrie::connect(&space)?;
        let (max_eid, attr_ents) = MaxEid::read(&trie)?.take(attrs.len());
        trie = max_eid.write(trie)?;
        let mut attr_map = attrs
            .iter()
            .cloned()
            .zip(attr_ents)
            .collect::<HashMap<_, _>>();
        attr_map.insert(Attr::DB_IDENT, Ent::DB_IDENT);
        for (at, a_ent) in &attr_map {
            let ident = Val::from(at);
            trie = add(trie, &attr_map, *a_ent, Attr::DB_IDENT, ident, &Txid::SETUP)?;
        }
        trie = set_max_tx(trie, Txid::FLOOR)?;
        trie.commit(&mut space)?;
        let db = Self {
            attr_map,
            trie: SpaceTrie::connect(&space)?,
            space,
        };
        Ok(db)
    }

    pub fn close(self) -> T {
        self.space
    }

    pub fn load(space: T, attrs: Vec<Attr>) -> Result<Self, LoadError> {
        let user_attrs: HashMap<String, Attr> = attrs
            .into_iter()
            .map(|attr| (attr.as_str().to_string(), attr))
            .collect();
        let trie = SpaceTrie::connect(&space)?;
        let attr_map = {
            let mut attr_map = HashMap::from([(Attr::DB_IDENT, Ent::DB_IDENT)]);
            let ents = {
                let mut rule = EntsWithAttr::new("e", Attr::DB_IDENT);
                rule.update(&trie, &attr_map)?;
                rule.results("e").to_vec()
            };
            for ent in ents {
                let mut rule = ValsWithEntAttr::new("v", ent, Attr::DB_IDENT);
                rule.update(&trie, &attr_map)?;
                let val = rule
                    .results("v")
                    .first()
                    .expect("val should exist")
                    .as_str();
                if let Some(user_attr) = user_attrs.get(val) {
                    attr_map.insert(*user_attr, ent);
                }
            }
            attr_map
        };
        // Confirm all requests attrs are present in the attr map.
        for attr in user_attrs.values() {
            if !attr_map.contains_key(attr) {
                return Err(LoadError::UnknownAttr(*attr));
            }
        }
        Ok(Self {
            attr_map,
            trie,
            space,
        })
    }
}

impl<T: Space> Db<T> {
    pub fn transact(self, datoms: Vec<Datom>) -> Result<Self, TransactError> {
        match datoms.is_empty() {
            true => Ok(self),
            false => {
                let tx = self.max_tx()?;
                let Self {
                    attr_map,
                    mut space,
                    mut trie,
                } = self;
                for datom in datoms {
                    trie = match datom {
                        Datom::Add(e, a, v) => add(trie, &attr_map, e, a, v, &tx)?,
                    }
                }
                trie = set_max_tx(trie, tx + 1)?;
                trie.commit(&mut space)?;
                let db = Self {
                    attr_map,
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

    pub fn max_eid(&self) -> Result<i32, QueryError> {
        let max_eid = MaxEid::read(&self.trie)?;
        Ok(max_eid.0)
    }
    pub fn find(&self, rule: &mut impl Rule) -> Result<bool, QueryError> {
        rule.update(&self.trie, &self.attr_map)
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
    attr_map: &HashMap<Attr, Ent>,
    e: Ent,
    a: Attr,
    v: Val,
    t: &Txid,
) -> Result<SpaceTrie<T>, TransactError> {
    let eid = e.to_id();
    let aid = attr_map.get(&a).expect("attr should exist").to_id();
    let (mut trie, vid) = val_table::insert(trie, v)?;
    let tid = t.u32();
    let eavt_key = [KEY_EAVT, eid, aid, vid.to_id()];
    let aevt_key = [KEY_AEVT, aid, eid, vid.to_id()];
    trie = trie.deep_insert(eavt_key, MemValue::from(tid))?;
    trie = trie.deep_insert(aevt_key, MemValue::from(tid))?;

    trie = MaxEid::read(&trie)?.update(trie, eid)?;
    Ok(trie)
}

fn set_max_tx<T: Space>(trie: SpaceTrie<T>, max_tx: Txid) -> Result<SpaceTrie<T>, TransactError> {
    trie.insert(KEY_MAX_TXID, MemValue::from(max_tx.u32()))
}
