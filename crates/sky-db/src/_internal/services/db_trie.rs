use crate::_internal::Vid;
use crate::_internal::datalog::Program;
use crate::_internal::datalog::atom::{Atom, atom};
use crate::_internal::datalog::rule::rule;
use crate::_internal::datalog::term::term;
use crate::_internal::datalog::var::var;
use crate::_internal::val_table;
use crate::_internal::{KEY_AEVT, KEY_EAVT, KEY_MAX_TXID};
use crate::Schema;
use crate::attr_table::AttrTable;
use crate::cardinality::Cardinality;
use crate::db;
use crate::trie::*;
use crate::trie::SkyKvsMut;
use crate::types::Txid;
use crate::types::txid;
use crate::{Attr, Dir, Ein, FindResult, TransactError, Val};
use async_stream::stream;
use futures::{StreamExt, pin_mut};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    trie: &mut SkyKvsMut,
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

pub(crate) async fn set_max_tx(trie: &mut SkyKvsMut, max_tx: Txid) -> Result<(), TransactError> {
    trie.insert(KEY_MAX_TXID, TrieValue::from(max_tx.u32()))
        .await;
    Ok(())
}

pub async fn find<'a, T>(
    trie: &'a T,
    schema: &'a Schema,
    select: impl Into<Vec<&'static str>>,
    where_: impl Into<Vec<Atom>>,
) -> FindResult
where
    T: QueryCursor + KvStream + Snap + Query,
{
    let select = select.into();
    let query_terms = select.iter().map(|s| term(var(*s))).collect::<Vec<_>>();
    let query_attr = db::query();
    let query_rule = rule(atom(query_attr.clone(), query_terms), where_.into());
    let program = Program::new([], [query_rule]);
    let kb = program.solve(trie, schema).await;
    let query_result = kb.query(query_attr);
    let mut found = FindResult::new();
    for row in query_result {
        let mut map = HashMap::new();
        if !row.is_empty() {
            let zipped = select
                .iter()
                .map(|s| s.to_string())
                .zip(row)
                .collect::<Vec<_>>();
            map.extend(zipped);
        }
        found.push(map);
    }
    found
}

pub fn ev_stream<T>(trie: &T, a: Attr, schema: &Schema) -> impl futures::Stream<Item = (i32, Val)>
where
    T: QueryCursor + KvStream + Snap + Query,
{
    stream! {
        let evt_subtrie = evt_subtrie(trie, a, schema).await;
        let evid_stream = evid_stream(evt_subtrie);
        pin_mut!(evid_stream);
        while let Some((eid, vid)) = evid_stream.next().await {
            let val = val_table::query(trie, Vid::from_id(vid)).await.ok().flatten().expect("val not found");
            yield (eid, val);
        }
    }
}

pub async fn list_entities<T>(trie: &T) -> Vec<Ein>
where
    T: QueryCursor + KvStream + Snap + Query,
{
    if let Some(root) = eavt_root(trie).await {
        root.query_all()
            .await
            .into_iter()
            .map(|(key, _)| Ein::from(key))
            .collect::<Vec<_>>()
    } else {
        vec![]
    }
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

pub async fn list_entity_attributes<T>(trie: &T, ein: Ein) -> Vec<AttrEin>
where
    T: QueryCursor + KvStream + Snap + Query,
{
    if let Some(root) = e_avt_subtrie(trie, ein).await {
        root.query_all()
            .await
            .into_iter()
            .map(|(key, _)| AttrEin::from(key))
            .collect::<Vec<_>>()
    } else {
        vec![]
    }
}

pub async fn list_entity_fills<T>(trie: &T, ein: Ein, attr_ein: AttrEin) -> Vec<(AttrEin, Vid)>
where
    T: QueryCursor + KvStream + Snap + Query,
{
    if let Some(root) = ea_vt_subtrie(trie, ein, attr_ein).await {
        root.query_all()
            .await
            .into_iter()
            .map(|(key, _)| (attr_ein, Vid::from_id(key)))
            .collect::<Vec<_>>()
    } else {
        vec![]
    }
}

async fn eavt_root<T>(trie: &T) -> Option<T::Snapshot>
where
    T: QueryCursor + KvStream + Snap + Query,
{
    let mut snapshot = trie.snapshot();
    snapshot.descend(KEY_EAVT).await;
    Some(snapshot)
}

async fn ea_vt_subtrie<T>(trie: &T, ein: Ein, attr_ein: AttrEin) -> Option<T::Snapshot>
where
    T: QueryCursor + KvStream + Snap + Query,
{
    let mut snapshot = trie.snapshot();
    snapshot
        .descend_n([KEY_EAVT, ein.to_i32(), attr_ein.to_i32()])
        .await;
    Some(snapshot)
}

async fn e_avt_subtrie<T>(trie: &T, ein: Ein) -> Option<T::Snapshot>
where
    T: QueryCursor + KvStream + Snap + Query,
{
    let mut snapshot = trie.snapshot();
    snapshot.descend_n([KEY_EAVT, ein.to_i32()]).await;
    Some(snapshot)
}

async fn evt_subtrie<T>(trie: &T, attr: Attr, schema: &Schema) -> T::Snapshot
where
    T: QueryCursor + KvStream + Snap + Query,
{
    let aid = schema[attr].ein().to_i32();

    let mut snapshot = trie.snapshot();
    snapshot.descend(KEY_AEVT).await;
    snapshot.descend(aid).await;
    snapshot
}

fn evid_stream<T>(evt_subtrie: T) -> impl futures::Stream<Item = (i32, i32)>
where
    T: QueryCursor + KvStream + Snap + Query,
{
    stream! {
        let evt_roots = evt_subtrie.map_base_stream();
        pin_mut!(evt_roots);
        while let Some((eid, _vt_trie)) = evt_roots.next().await {
            let mut vt_subtrie = evt_subtrie.snapshot();
            vt_subtrie.descend(eid).await;

            let vt_stream = vt_subtrie.u32_stream();
            pin_mut!(vt_stream);
            while let Some((vid, tx_u32)) = vt_stream.next().await {
                let tx_value = Value::from(tx_u32);
                if tx_value.dir == Dir::In {
                    yield (eid, vid);
                }
            }
        }
    }
}
