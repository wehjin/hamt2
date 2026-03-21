use crate::db::datom::{Attr, Datom, Ent, Val};
use crate::db::txid::Txid;
use crate::hamt::space::mem::MemSpace;
use crate::hamt::trie::mem::value::MemValue;
use crate::hamt::trie::space::SpaceTrie;
use crate::{QueryError, TransactError};

const KEY_MAX_TXID: i32 = -1;
const KEY_EAVT: i32 = -2;
const KEY_AEVT: i32 = -3;

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
    pub fn pull(&self, e: Ent, a: Attr) -> Result<Option<Val>, QueryError> {
        let trie = &self.trie;
        let eavt_key = [KEY_EAVT, e.i32(), a.to_ent().i32()];
        let eavt_value = trie.deep_query_value(eavt_key)?;
        let value = if let Some(MemValue::U32(value)) = eavt_value {
            Some(Val::U32(value))
        } else {
            None
        };
        Ok(value)
    }
}

fn add(
    mut trie: SpaceTrie,
    e: Ent,
    a: Attr,
    v: Val,
    _t: &Txid,
) -> Result<SpaceTrie, TransactError> {
    let a_ent = a.to_ent();
    let eavt_key = [KEY_EAVT, e.i32(), a_ent.i32()];
    let aevt_key = [KEY_AEVT, a_ent.i32(), e.i32()];
    trie = trie.deep_insert(eavt_key, MemValue::from(v.u32()))?;
    trie = trie.deep_insert(aevt_key, MemValue::from(v.u32()))?;
    Ok(trie)
}

fn set_max_tx(trie: SpaceTrie, max_tx: Txid) -> Result<SpaceTrie, TransactError> {
    trie.insert(KEY_MAX_TXID, MemValue::from(max_tx.u32()))
}