use crate::db::datom::{Attr, Datom, Ent, Val};
use crate::db::find::EntsWithAttr;
use crate::db::find::Rule;
use crate::db::find::ValsWithEntAttr;
use crate::db::txid::Txid;
use crate::space::mem::MemSpace;
use crate::hamt::trie::mem::value::MemValue;
use crate::hamt::trie::space::SpaceTrie;
use crate::{LoadError, QueryError, TransactError};
use std::collections::HashMap;

pub(crate) const KEY_MAX_TXID: i32 = -1;
pub(crate) const KEY_MAX_VID: i32 = -2;
pub(crate) const KEY_VALS: i32 = -3;
pub(crate) const KEY_EAVT: i32 = -4;
pub(crate) const KEY_AEVT: i32 = -5;
pub(crate) const KEY_MAX_EID: i32 = -6;

struct MaxEid(i32);
impl MaxEid {
    fn take(self, count: usize) -> (Self, Vec<Ent>) {
        let Self(start) = self;
        let end = start + count as i32;
        let ids = (start..end).map(Ent).collect();
        (Self(end), ids)
    }
    fn update(mut self, trie: SpaceTrie, eid: i32) -> Result<SpaceTrie, TransactError> {
        if eid < self.0 {
            Ok(trie)
        } else {
            self.0 = eid + 1;
            self.write(trie)
        }
    }
    fn write(self, trie: SpaceTrie) -> Result<SpaceTrie, TransactError> {
        let trie = trie.insert(KEY_MAX_EID, MemValue::from(self.0 as u32))?;
        Ok(trie)
    }
    fn read(trie: &SpaceTrie) -> Result<Self, QueryError> {
        if let Some(MemValue::U32(value)) = trie.query_value(KEY_MAX_EID)? {
            Ok(Self(value as i32))
        } else {
            Ok(Self(0))
        }
    }
}

pub struct Db {
    attr_map: HashMap<Attr, Ent>,
    trie: SpaceTrie,
    space: MemSpace,
}

impl Db {
    pub fn new(attrs: Vec<Attr>) -> Result<Self, TransactError> {
        let mut space = MemSpace::new();
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
    pub fn close(self) -> MemSpace {
        self.space
    }
    pub fn load(space: MemSpace, attrs: Vec<Attr>) -> Result<Self, LoadError> {
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

impl Db {
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
                Ok(Self {
                    attr_map,
                    trie: SpaceTrie::connect(&space)?,
                    space,
                })
            }
        }
    }
}

impl Db {
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

fn add(
    trie: SpaceTrie,
    attr_map: &HashMap<Attr, Ent>,
    e: Ent,
    a: Attr,
    v: Val,
    t: &Txid,
) -> Result<SpaceTrie, TransactError> {
    let eid = e.to_id();
    let aid = attr_map.get(&a).expect("attr should exist").to_id();
    let (vid, mut trie) = add_val(trie, v)?;
    let tid = t.u32();
    let eavt_key = [KEY_EAVT, eid, aid, vid];
    let aevt_key = [KEY_AEVT, aid, eid, vid];
    trie = trie.deep_insert(eavt_key, MemValue::from(tid))?;
    trie = trie.deep_insert(aevt_key, MemValue::from(tid))?;
    trie = MaxEid::read(&trie)?.update(trie, eid)?;
    Ok(trie)
}

fn add_val(mut trie: SpaceTrie, v: Val) -> Result<(i32, SpaceTrie), TransactError> {
    let vid = max_vid(&trie)?;
    let max_vid = vid + 1;
    let value = mem_value(v);
    trie = trie.deep_insert([KEY_VALS, vid], value)?;
    trie = set_max_vid(trie, max_vid)?;
    Ok((vid, trie))
}

fn set_max_vid(trie: SpaceTrie, max_vid: i32) -> Result<SpaceTrie, TransactError> {
    trie.insert(KEY_MAX_VID, MemValue::from(max_vid as u32))
}

fn max_vid(trie: &SpaceTrie) -> Result<i32, QueryError> {
    if let Some(MemValue::U32(value)) = trie.query_value(KEY_MAX_VID)? {
        Ok(value as i32)
    } else {
        Ok(0)
    }
}
fn set_max_tx(trie: SpaceTrie, max_tx: Txid) -> Result<SpaceTrie, TransactError> {
    trie.insert(KEY_MAX_TXID, MemValue::from(max_tx.u32()))
}

fn mem_value(v: Val) -> MemValue {
    match v {
        Val::U32(value) => MemValue::from(value),
        Val::String(value) => MemValue::from(value),
    }
}
