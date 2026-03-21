use crate::db::datom::{Attr, Datom, Ent, Val};
use crate::db::txid::Txid;
use crate::hamt::space::mem::MemSpace;
use crate::hamt::trie::mem::value::MemValue;
use crate::hamt::trie::space::SpaceTrie;
use crate::{QueryError, TransactError};

const KEY_MAX_TXID: i32 = -1;
const KEY_MAX_VID: i32 = -2;
const KEY_VALS: i32 = -3;
const KEY_EAVT: i32 = -4;
const KEY_AEVT: i32 = -5;

pub struct Db {
    trie: SpaceTrie,
    space: MemSpace,
}

impl Db {
    pub fn new() -> Result<Self, TransactError> {
        let mut space = MemSpace::new();
        let mut setup = SpaceTrie::connect(&space)?;
        setup = set_max_tx(setup, Txid::FLOOR)?;
        setup.commit(&mut space)?;
        Ok(Self {
            trie: SpaceTrie::connect(&space)?,
            space,
        })
    }

    pub fn max_tx(&self) -> Result<Txid, QueryError> {
        let Some(MemValue::U32(value)) = self.trie.query_value(KEY_MAX_TXID)? else {
            panic!("max_tx not found");
        };
        Ok(Txid::from(value))
    }

    pub fn transact(self, datoms: Vec<Datom>) -> Result<Self, TransactError> {
        match datoms.is_empty() {
            true => Ok(self),
            false => {
                let tx = self.max_tx()?;
                let Self {
                    mut space,
                    mut trie,
                } = self;
                for datom in datoms {
                    trie = match datom {
                        Datom::Add(e, a, v) => add(trie, e, a, v, &tx)?,
                    }
                }
                trie = set_max_tx(trie, tx + 1)?;
                trie.commit(&mut space)?;
                Ok(Self {
                    trie: SpaceTrie::connect(&space)?,
                    space,
                })
            }
        }
    }
    pub fn find_val(&self, e: Ent, a: Attr) -> Result<Option<Val>, QueryError> {
        let trie = &self.trie;
        let eavt_key = [KEY_EAVT, e.to_id(), a.to_id()];
        let eavt_value = trie.deep_query_value(eavt_key)?;
        let value = if let Some(MemValue::MapBase(map_base)) = eavt_value {
            let vid = trie
                .to_subtrie(map_base)
                .query_key_values()?
                .first()
                .map(|(vid, _)| *vid)
                .expect("vid should exist");
            let val = val(&trie, vid)?.expect("val should exist");
            Some(val)
        } else {
            None
        };
        Ok(value)
    }
}

fn add(trie: SpaceTrie, e: Ent, a: Attr, v: Val, t: &Txid) -> Result<SpaceTrie, TransactError> {
    let eid = e.to_id();
    let aid = a.to_id();
    let (vid, mut trie) = add_val(trie, v)?;
    let tid = t.u32();
    let eavt_key = [KEY_EAVT, eid, aid, vid];
    let aevt_key = [KEY_AEVT, aid, eid, vid];
    trie = trie.deep_insert(eavt_key, MemValue::from(tid))?;
    trie = trie.deep_insert(aevt_key, MemValue::from(tid))?;
    Ok(trie)
}

fn add_val(mut trie: SpaceTrie, v: Val) -> Result<(i32, SpaceTrie), TransactError> {
    let vid = max_vid(&trie)?;
    let max_vid = vid + 1;
    let value = mem_value(&v);
    trie = trie.deep_insert([KEY_VALS, vid], value)?;
    trie = set_max_vid(trie, max_vid)?;
    Ok((vid, trie))
}

fn val(trie: &SpaceTrie, vid: i32) -> Result<Option<Val>, QueryError> {
    match trie.deep_query_value([KEY_VALS, vid])? {
        Some(value) => Ok(Some(Val::from(value))),
        None => Ok(None),
    }
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

fn mem_value(v: &Val) -> MemValue {
    match v {
        Val::U32(value) => MemValue::from(*value),
    }
}
