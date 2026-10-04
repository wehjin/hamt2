use crate::_internal::Vid;
use crate::_internal::val_table;
use crate::_internal::{KEY_AEVT, KEY_EAVT, KEY_MAX_TXID};
use crate::Schema;
use crate::attr_table::AttrTable;
use crate::cardinality::Cardinality;
use crate::trie::SkyTrieMut;
use crate::trie::*;
use crate::types::Txid;
use crate::types::txid;
use crate::{Attr, Dir, Ein, TransactError, Val};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Value {
    pub id: Txid,
    pub dir: Dir,
}
impl Into<TrieValue> for Value {
    fn into(self) -> TrieValue {
        debug_assert!(self.id.u32() <= 0x0FFF_FFFF);
        let id_part = 0x0FFF_FFFF & self.id.u32();
        let dir_part = match self.dir {
            Dir::Out => 0x0000_0000,
            Dir::In => 0x1000_0000,
        };
        let combined = id_part | dir_part;
        TrieValue::U32(combined)
    }
}

impl From<u32> for Value {
    fn from(combined: u32) -> Self {
        let id = txid(combined & 0x0FFF_FFFF);
        let dir = if (combined & 0xF000_0000) == 0x1000_0000 {
            Dir::In
        } else {
            Dir::Out
        };
        Value { id, dir }
    }
}

pub(crate) async fn with_update(
    trie: &mut SkyTrieMut,
    attr_map: &AttrTable,
    ein: Ein,
    attr: Attr,
    val: Val,
    dir: Dir,
    txid: &Txid,
) -> Result<(), TransactError> {
    let attribute = &attr_map[attr];
    let eid = ein.to_i32();
    let aid = attribute.ein().to_i32();
    let vid = val_table::insert(trie, val).await?;
    let eavt_key = [KEY_EAVT, eid, aid, vid.to_id()];
    let aevt_key = [KEY_AEVT, aid, eid, vid.to_id()];
    let replace_tail = attribute.cardinality() == Cardinality::One;
    let tx_value = Value { id: *txid, dir };
    trie.insert_deep(eavt_key, tx_value, replace_tail).await;
    trie.insert_deep(aevt_key, tx_value, replace_tail).await;
    Ok(())
}

pub(crate) async fn set_max_tx(trie: &mut SkyTrieMut, max_tx: Txid) -> Result<(), TransactError> {
    trie.insert(KEY_MAX_TXID, TrieValue::from(max_tx.u32()))
        .await;
    Ok(())
}

pub fn list_entities<T>(trie: &T) -> Vec<Ein>
where
    T: QueryCursor + Snap + Query,
{
    eavt_root(trie)
        .query_all()
        .into_iter()
        .map(|(key, _)| Ein::from(key))
        .collect::<Vec<_>>()
}

/// An attr-ein is an Ein that refers to an attribute.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct AttrEin(Ein);
impl AttrEin {
    pub fn ein(&self) -> Ein {
        self.0
    }

    pub fn to_i32(&self) -> i32 {
        self.0.to_i32()
    }
}
impl From<i32> for AttrEin {
    fn from(ein: i32) -> Self {
        AttrEin(Ein(ein))
    }
}

pub fn list_entity_attributes<T>(trie: &T, ein: Ein) -> Vec<AttrEin>
where
    T: QueryCursor + Snap + Query,
{
    e_avt_subtrie(trie, ein)
        .query_all()
        .into_iter()
        .map(|(key, _)| AttrEin::from(key))
        .collect::<Vec<_>>()
}

pub fn list_entity_fills<T>(trie: &T, ein: Ein, attr_ein: AttrEin) -> Vec<(AttrEin, Vid)>
where
    T: QueryCursor + Snap + Query,
{
    ea_vt_subtrie(trie, ein, attr_ein)
        .query_all()
        .into_iter()
        .map(|(key, _)| (attr_ein, Vid::from_id(key)))
        .collect::<Vec<_>>()
}

fn eavt_root<T>(trie: &T) -> T::Snapshot
where
    T: QueryCursor + Snap + Query,
{
    let mut snapshot = trie.snapshot();
    snapshot.descend(KEY_EAVT);
    snapshot
}

fn ea_vt_subtrie<T>(trie: &T, ein: Ein, attr_ein: AttrEin) -> T::Snapshot
where
    T: QueryCursor + Snap + Query,
{
    let mut snapshot = trie.snapshot();
    snapshot.descend_n([KEY_EAVT, ein.to_i32(), attr_ein.to_i32()]);
    snapshot
}

fn e_avt_subtrie<T>(trie: &T, ein: Ein) -> T::Snapshot
where
    T: QueryCursor + Snap + Query,
{
    let mut snapshot = trie.snapshot();
    snapshot.descend_n([KEY_EAVT, ein.to_i32()]);
    snapshot
}

pub fn evid_iter(evt_subtrie: SkyTrie) -> impl Iterator<Item = (i32, i32)> {
    let template = evt_subtrie.clone();
    evt_subtrie
        .into_iter()
        .map_bases()
        .flat_map(move |(eid, _)| {
            let mut vt = template.clone();
            vt.descend(eid);
            vt.into_iter()
                .u32s()
                .filter_map(move |(vid, tx)| (Value::from(tx).dir == Dir::In).then_some((eid, vid)))
        })
}

pub fn ev_iter(
    trie: &SkyTrie,
    a: Attr,
    schema: &Schema,
) -> impl Iterator<Item = (i32, Val)> {
    let aid = schema[a].ein().to_i32();
    let mut evt = trie.clone();
    evt.descend(KEY_AEVT);
    evt.descend(aid);
    evid_iter(evt).map(move |(eid, vid)| {
        let val = val_table::query(trie, Vid::from_id(vid))
            .ok()
            .flatten()
            .expect("val not found");
        (eid, val)
    })
}
