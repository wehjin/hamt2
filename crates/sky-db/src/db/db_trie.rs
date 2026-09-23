use crate::crate_services::datalog::Program;
use crate::crate_services::datalog::atom::{Atom, atom};
use crate::crate_services::datalog::rule::rule;
use crate::crate_services::datalog::term::term;
use crate::crate_services::datalog::var::var;
use crate::crate_services::val_table;
use crate::db::Schema;
use crate::db::attr_table::AttrTable;
use crate::db::cardinality::Cardinality;
use crate::db::types::key::{KEY_AEVT, KEY_EAVT, KEY_MAX_TXID};
use crate::db::vid::Vid;
use crate::db::{Txid, txid};
use crate::trie::prelude::*;
use async_stream::stream;
use futures::{StreamExt, pin_mut};
use serde::{Deserialize, Serialize};
use sky_types::db;
use sky_types::db::{Attr, Dir, Ein, FindResult, TransactError, Val};
use sky_types::storage::ReadWriteStorage;
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

pub(crate) async fn with_update<S: ReadWriteStorage>(
    trie: Trie<S>,
    attr_map: &AttrTable,
    ein: Ein,
    attr: Attr,
    val: Val,
    dir: Dir,
    txid: &Txid,
) -> Result<Trie<S>, TransactError> {
    let attribute = &attr_map[attr];
    let eid = ein.to_i32();
    let aid = attribute.ein().to_i32();
    let (mut trie, vid) = val_table::insert(trie, val).await?;
    let eavt_key = [KEY_EAVT, eid, aid, vid.to_id()];
    let aevt_key = [KEY_AEVT, aid, eid, vid.to_id()];
    let replace_tail = attribute.cardinality() == Cardinality::One;
    let tx_value = Value { id: *txid, dir };
    trie.deep_insert(eavt_key, tx_value, replace_tail).await?;
    trie.deep_insert(aevt_key, tx_value, replace_tail).await?;
    Ok(trie)
}

pub(crate) async fn set_max_tx<S: ReadWriteStorage>(
    mut trie: Trie<S>,
    max_tx: Txid,
) -> Result<Trie<S>, TransactError> {
    trie.insert(KEY_MAX_TXID, TrieValue::from(max_tx.u32()))
        .await?;
    Ok(trie)
}

pub async fn find<'a, T>(
    trie: &'a T,
    schema: &'a Schema,
    select: impl Into<Vec<&'static str>>,
    where_: impl Into<Vec<Atom>>,
) -> FindResult
where
    T: TrieStream,
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

pub fn ev_stream<'a, T>(
    trie: &'a T,
    a: Attr,
    schema: &'a Schema,
) -> impl futures::Stream<Item = (i32, Val)> + 'a
where
    T: TrieStream + 'a,
{
    stream! {
        if let Some(evt_subtrie) = evt_subtrie(trie, a, schema).await {
            let evid_stream = evid_stream(evt_subtrie);
            pin_mut!(evid_stream);
            while let Some((eid, vid)) = evid_stream.next().await {
                let val = val_table::query(trie, Vid::from_id(vid)).await.ok().flatten().expect("val not found");
                yield (eid, val);
            }
        }
    }
}

pub async fn list_entities<T>(trie: &T) -> Vec<Ein>
where
    T: TrieStream,
{
    if let Some(root) = eavt_root(trie).await {
        root.query_keys_values()
            .await
            .expect("read keys and values from evt root")
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
}
impl From<i32> for AttrEin {
    fn from(ein: i32) -> Self {
        AttrEin(Ein(ein))
    }
}

pub async fn list_entity_attributes<T>(trie: &T, ein: Ein) -> Vec<AttrEin>
where
    T: TrieStream,
{
    if let Some(root) = e_avt_subtrie(trie, ein).await {
        root.query_keys_values()
            .await
            .expect("read keys and values from trie")
            .into_iter()
            .map(|(key, _)| AttrEin::from(key))
            .collect::<Vec<_>>()
    } else {
        vec![]
    }
}

async fn eavt_root<T>(trie: &T) -> Option<T::Subtrie>
where
    T: TrieStream,
{
    let root_value = trie.deep_query_value([KEY_EAVT]).await.ok().flatten();
    root_value.and_then(|value| trie.to_subtrie_in_value(value))
}

async fn e_avt_subtrie<T>(trie: &T, ein: Ein) -> Option<T::Subtrie>
where
    T: TrieStream,
{
    let root_value = trie
        .deep_query_value([KEY_EAVT, ein.to_i32()])
        .await
        .ok()
        .flatten();
    root_value.and_then(|value| trie.to_subtrie_in_value(value))
}

async fn evt_subtrie<T>(trie: &T, attr: Attr, schema: &Schema) -> Option<T::Subtrie>
where
    T: TrieStream,
{
    let aid = schema[attr].ein().to_i32();
    let keys = [KEY_AEVT, aid];
    let evt_value = trie.deep_query_value(keys).await.ok().flatten();
    evt_value.and_then(|evt| trie.to_subtrie_in_value(evt))
}

fn evid_stream<T: TrieStream>(evt_subtrie: T) -> impl futures::Stream<Item = (i32, i32)> {
    stream! {
        let evt_stream = evt_subtrie.subtrie_stream();
        pin_mut!(evt_stream);
        while let Some((eid, vt_trie)) = evt_stream.next().await {
            let vt_stream = vt_trie.u32_stream();
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
