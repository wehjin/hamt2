use crate::_internal::Vid;
use crate::_internal::val_table;
use crate::_internal::{KEY_AEVT, KEY_EAVT, KEY_MAX_TXID};
use crate::Schema;
use crate::attr_table::AttrTable;
use crate::cardinality::Cardinality;
use crate::trie::SkyTrieMut;
use crate::trie::*;
use crate::types::Txid;
use crate::{Attr, Dir, Ein, TransactError, Val};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

fn dir_to_u32(dir: Dir) -> u32 {
    match dir {
        Dir::In => 1,
        Dir::Out => 0,
    }
}

fn dir_from_u32(u: u32) -> Dir {
    match u {
        1 => Dir::In,
        _ => Dir::Out,
    }
}

/// Resolve the "current" values of a single (entity, attr) slot from its full
/// history of (value, tx, dir) entries.
///
/// Cardinality-one keeps only the single value written by the latest tx (if it
/// was an add); cardinality-many keeps every value whose latest tx was an add.
fn current_vids(entries: Vec<(Vid, Txid, Dir)>, card: Cardinality) -> Vec<Vid> {
    match card {
        Cardinality::One => entries
            .into_iter()
            .max_by_key(|(_, tx, _)| *tx)
            .filter(|(_, _, dir)| *dir == Dir::In)
            .map(|(vid, _, _)| vec![vid])
            .unwrap_or_default(),
        Cardinality::Many => {
            let mut latest: BTreeMap<Vid, (Txid, Dir)> = BTreeMap::new();
            for (vid, tx, dir) in entries {
                match latest.entry(vid) {
                    Entry::Vacant(e) => {
                        e.insert((tx, dir));
                    }
                    Entry::Occupied(mut e) => {
                        if tx > e.get().0 {
                            *e.get_mut() = (tx, dir);
                        }
                    }
                }
            }
            latest
                .into_iter()
                .filter(|(_, (_, dir))| *dir == Dir::In)
                .map(|(vid, _)| vid)
                .collect()
        }
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
    let tx = txid.u32() as i32;
    let eavt_key = [KEY_EAVT, eid, aid, vid.to_id(), tx];
    let aevt_key = [KEY_AEVT, aid, eid, vid.to_id(), tx];
    let dir_val = TrieValue::U32(dir_to_u32(dir));
    trie.insert_deep(eavt_key, dir_val.clone(), false).await;
    trie.insert_deep(aevt_key, dir_val, false).await;
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

pub fn list_entity_fills<T>(
    trie: &T,
    ein: Ein,
    attr_ein: AttrEin,
    cardinality: Cardinality,
) -> Vec<(AttrEin, Vid)>
where
    T: QueryCursor + Snap + Query,
{
    let entries = collect_entries_eavt(trie, ein, attr_ein);
    current_vids(entries, cardinality)
        .into_iter()
        .map(|vid| (attr_ein, vid))
        .collect()
}

fn collect_entries_eavt<T>(trie: &T, ein: Ein, attr_ein: AttrEin) -> Vec<(Vid, Txid, Dir)>
where
    T: QueryCursor + Snap + Query,
{
    let mut entries = Vec::new();
    let mut vt = trie.snapshot();
    vt.descend_n([KEY_EAVT, ein.to_i32(), attr_ein.to_i32()]);
    for (vid, _) in vt.query_all() {
        let mut vt2 = trie.snapshot();
        vt2.descend_n([KEY_EAVT, ein.to_i32(), attr_ein.to_i32(), vid]);
        for (txid, dir_val) in vt2.query_all() {
            if let TrieValue::U32(dir) = dir_val {
                entries.push((Vid::from_id(vid), Txid::from(txid as u32), dir_from_u32(dir)));
            }
        }
    }
    entries
}

fn eavt_root<T>(trie: &T) -> T::Snapshot
where
    T: QueryCursor + Snap + Query,
{
    let mut snapshot = trie.snapshot();
    snapshot.descend(KEY_EAVT);
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

pub fn ev_iter(trie: &SkyTrie, a: Attr, schema: &Schema) -> impl Iterator<Item = (i32, Val)> {
    let aid = schema[a.clone()].ein().to_i32();
    let cardinality = schema[a].cardinality();
    let mut result = Vec::new();
    let mut evt = trie.clone();
    evt.descend(KEY_AEVT);
    evt.descend(aid);
    for (eid, _) in evt.query_all() {
        let mut entries = Vec::new();
        let mut vt = trie.clone();
        vt.descend(KEY_AEVT);
        vt.descend(aid);
        vt.descend(eid);
        for (vid, _) in vt.query_all() {
            let mut vt2 = trie.clone();
            vt2.descend(KEY_AEVT);
            vt2.descend(aid);
            vt2.descend(eid);
            vt2.descend(vid);
            for (txid, dir_val) in vt2.query_all() {
                if let TrieValue::U32(dir) = dir_val {
                    entries.push((Vid::from_id(vid), Txid::from(txid as u32), dir_from_u32(dir)));
                }
            }
        }
        for vid in current_vids(entries, cardinality) {
            let val = val_table::query(trie, vid)
                .ok()
                .flatten()
                .expect("val not found");
            result.push((eid, val));
        }
    }
    result.into_iter()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attr_spec::AttrSpec;
    use crate::attribute::AttributeDetails;
    use crate::trie::SkyTrie;
    use crate::val;

    fn attr_table() -> AttrTable {
        let mut t = AttrTable::starter();
        t.insert(AttributeDetails::new(
            Ein(100),
            AttrSpec {
                attr: Attr::from("counter/count"),
                cardinality: Cardinality::One,
            },
        ));
        t
    }

    #[tokio::test]
    async fn history_is_retained() {
        let attr = Attr::from("counter/count");
        let table = attr_table();
        let mut trie = SkyTrie::new();
        trie.edit(async |trie| {
            with_update(trie, &table, Ein(100), attr.clone(), val(100), Dir::In, &Txid::from(1u32))
                .await?;
            with_update(trie, &table, Ein(100), attr.clone(), val(101), Dir::In, &Txid::from(2u32))
                .await?;
            Ok(())
        })
        .await
        .unwrap();

        let fills = list_entity_fills(&trie, Ein(100), AttrEin::from(100), Cardinality::One);
        assert_eq!(fills.len(), 1);

        let mut vt = trie.clone();
        vt.descend(KEY_EAVT);
        vt.descend(100);
        vt.descend(100);
        assert_eq!(vt.query_all().len(), 2);
    }
}
