use crate::db::attr_table::AttrTable;
use crate::db::component::key::{KEY_AEVT, KEY_EAVT, KEY_MAX_TXID};
use crate::db::component::val_table;
use crate::db::db::QUERY;
use crate::db::find::program::atom::{atom, Atom};
use crate::db::find::program::rule::rule;
use crate::db::find::program::term::term;
use crate::db::find::program::var::var;
use crate::db::find::program::Program;
use crate::db::find_result::FindResult;
use crate::db::{Attr, Txid, Val, Vid};
use crate::db::{Ein, Schema};
use crate::space::Space;
use crate::trie::mem::value::MemValue;
use crate::trie::SpaceTrie;
use crate::TransactError;
use async_stream::stream;
use futures::{pin_mut, StreamExt};
use std::collections::HashMap;

pub(crate) async fn add<T: Space>(
    trie: SpaceTrie<T>,
    attr_map: &AttrTable,
    ein: Ein,
    attr: Attr,
    val: Val,
    t: &Txid,
) -> Result<SpaceTrie<T>, TransactError> {
    let eid = ein.to_i32();
    let aid = attr_map[attr].ein().to_i32();
    let (mut trie, vid) = val_table::insert(trie, val).await?;
    let tid = t.u32();
    let eavt_key = [KEY_EAVT, eid, aid, vid.to_id()];
    let aevt_key = [KEY_AEVT, aid, eid, vid.to_id()];
    trie = trie.deep_insert(eavt_key, MemValue::from(tid)).await?;
    trie = trie.deep_insert(aevt_key, MemValue::from(tid)).await?;
    Ok(trie)
}

pub(crate) async fn set_max_tx<T: Space>(
    trie: SpaceTrie<T>,
    max_tx: Txid,
) -> Result<SpaceTrie<T>, TransactError> {
    trie.insert(KEY_MAX_TXID, MemValue::from(max_tx.u32()))
        .await
}

pub async fn find<T: Space>(
    trie: &SpaceTrie<T>,
    schema: &Schema,
    select: impl Into<Vec<&'static str>>,
    where_: impl Into<Vec<Atom>>,
) -> FindResult {
    let select = select.into();
    let query_terms = select.iter().map(|s| term(var(*s))).collect::<Vec<_>>();
    let query_rule = rule(atom(QUERY, query_terms), where_.into());
    let program = Program::new([], [query_rule]);
    let kb = program.solve(trie, schema).await;
    let query_result = kb.query(QUERY);
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

pub fn ev_stream<T: Space>(
    trie: &SpaceTrie<T>,
    a: Attr,
    schema: &Schema,
) -> impl futures::Stream<Item = (i32, Val)> {
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

async fn evt_subtrie<T: Space>(
    trie: &SpaceTrie<T>,
    attr: Attr,
    schema: &Schema,
) -> Option<SpaceTrie<T>> {
    let aid = schema[attr].ein().to_i32();
    let keys = [KEY_AEVT, aid];
    let evt_value = trie.deep_query_value(keys).await.ok().flatten();
    if let Some(evt) = evt_value {
        trie.to_subtrie_from_value(evt).await.ok()
    } else {
        None
    }
}

fn evid_stream<T: Space>(
    evt_subtrie: SpaceTrie<T>,
) -> impl futures::Stream<Item = (i32, i32)> + use<T> {
    stream! {
        let evt_stream = evt_subtrie.subtrie_stream();
        pin_mut!(evt_stream);
        while let Some((eid, vt_trie)) = evt_stream.next().await {
            let vt_stream = vt_trie.u32_stream();
            pin_mut!(vt_stream);
            if let Some((vid, _)) = vt_stream.next().await {
                yield (eid, vid);
            }
        }
    }
}
