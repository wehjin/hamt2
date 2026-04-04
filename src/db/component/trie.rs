use crate::db::component::val_table;
use crate::db::component::key::{KEY_AEVT, KEY_EAVT, KEY_MAX_TXID};
use crate::db::{Attr, Txid, Val};
use crate::space::Space;
use crate::trie::mem::value::MemValue;
use crate::trie::SpaceTrie;
use crate::TransactError;
use std::collections::HashMap;
use crate::db::Eid;

pub(crate) async fn trie_add<T: Space>(
    trie: SpaceTrie<T>,
    attr_map: &HashMap<Attr, Eid>,
    e: Eid,
    a: Attr,
    v: Val,
    t: &Txid,
) -> Result<SpaceTrie<T>, TransactError> {
    let eid = e.to_i32();
    let aid = attr_map.get(&a).expect("attr should exist").to_i32();
    let (mut trie, vid) = val_table::insert(trie, v).await?;
    let tid = t.u32();
    let eavt_key = [KEY_EAVT, eid, aid, vid.to_id()];
    let aevt_key = [KEY_AEVT, aid, eid, vid.to_id()];
    trie = trie.deep_insert(eavt_key, MemValue::from(tid)).await?;
    trie = trie.deep_insert(aevt_key, MemValue::from(tid)).await?;
    Ok(trie)
}

pub(crate) async fn trie_set_max_tx<T: Space>(
    trie: SpaceTrie<T>,
    max_tx: Txid,
) -> Result<SpaceTrie<T>, TransactError> {
    trie.insert(KEY_MAX_TXID, MemValue::from(max_tx.u32()))
        .await
}
