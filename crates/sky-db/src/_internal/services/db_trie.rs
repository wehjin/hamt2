use crate::_internal::Vid;
use crate::_internal::val_table;
use crate::_internal::{KEY_AEVT, KEY_EAVT};
use crate::Schema;
use crate::cardinality::Cardinality;
use crate::trie::*;
use crate::{Attr, Dir, Ein, Fact, TransactError, Val};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::btree_map::Entry;

/// Returns the trie at the deep path `path`, or an empty trie if absent.
pub(crate) fn subtrie_at<const N: usize>(trie: &SkyTrie, path: [i32; N]) -> SkyTrie {
    match TrieValue::Trie(trie.clone()).query_deep(path) {
        TrieValue::Trie(t) => t,
        _ => SkyTrie::new(),
    }
}

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
/// "Latest" is determined by the linearized position in `order`.
fn current_vids(
    entries: Vec<(Vid, Ein, Dir)>,
    card: Cardinality,
    order: &HashMap<Ein, u32>,
) -> Vec<Vid> {
    match card {
        Cardinality::One => entries
            .into_iter()
            .max_by_key(|(_, tx_ein, _)| order[tx_ein])
            .filter(|(_, _, dir)| *dir == Dir::In)
            .map(|(vid, _, _)| vec![vid])
            .unwrap_or_default(),
        Cardinality::Many => {
            let mut latest: BTreeMap<Vid, (u32, Dir)> = BTreeMap::new();
            for (vid, tx_ein, dir) in entries {
                let pos = order[&tx_ein];
                match latest.entry(vid) {
                    Entry::Vacant(e) => {
                        e.insert((pos, dir));
                    }
                    Entry::Occupied(mut e) => {
                        if pos > e.get().0 {
                            *e.get_mut() = (pos, dir);
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

pub(crate) fn with_update(
    trie: &SkyTrie,
    aid: Ein,
    ein: Ein,
    val: Val,
    dir: Dir,
    tx_ein: Ein,
) -> Result<SkyTrie, TransactError> {
    let eid = ein.to_i32();
    let aid = aid.to_i32();
    let (trie, vid) = val_table::insert(trie, val)?;
    let tx = tx_ein.to_i32();
    let eavt_key = [KEY_EAVT, eid, aid, vid.to_id(), tx];
    let aevt_key = [KEY_AEVT, aid, eid, vid.to_id(), tx];
    let dir_val = TrieValue::U32(dir_to_u32(dir));
    let TrieValue::Trie(trie) = TrieValue::Trie(trie)
        .insert_deep(eavt_key, dir_val.clone())
        .insert_deep(aevt_key, dir_val)
    else {
        unreachable!("inserting into a trie yields a trie");
    };
    Ok(trie)
}

pub fn list_entities(trie: &SkyTrie) -> Vec<Ein> {
    eavt_root(trie)
        .query_all()
        .into_iter()
        .map(|(key, _)| Ein::from(key))
        .collect::<Vec<_>>()
}

pub fn list_entity_attributes(trie: &SkyTrie, ein: Ein) -> Vec<Ein> {
    e_avt_subtrie(trie, ein)
        .query_all()
        .into_iter()
        .map(|(key, _)| Ein::from(key))
        .collect::<Vec<_>>()
}

pub fn list_entity_fills(
    trie: &SkyTrie,
    ein: Ein,
    attr_ein: Ein,
    cardinality: Cardinality,
    order: &HashMap<Ein, u32>,
) -> Vec<(Ein, Vid)> {
    let entries = collect_entries_eavt(trie, ein, attr_ein);
    current_vids(entries, cardinality, order)
        .into_iter()
        .map(|vid| (attr_ein, vid))
        .collect()
}

fn collect_entries_eavt(trie: &SkyTrie, ein: Ein, attr_ein: Ein) -> Vec<(Vid, Ein, Dir)> {
    let mut entries = Vec::new();
    let vt = subtrie_at(trie, [KEY_EAVT, ein.to_i32(), attr_ein.to_i32()]);
    for (vid, _) in vt.query_all() {
        let vt2 = subtrie_at(trie, [KEY_EAVT, ein.to_i32(), attr_ein.to_i32(), vid]);
        for (tx_ein, dir_val) in vt2.query_all() {
            if let TrieValue::U32(dir) = dir_val {
                entries.push((Vid::from_id(vid), Ein::from(tx_ein), dir_from_u32(dir)));
            }
        }
    }
    entries
}

fn eavt_root(trie: &SkyTrie) -> SkyTrie {
    subtrie_at(trie, [KEY_EAVT])
}

fn e_avt_subtrie(trie: &SkyTrie, ein: Ein) -> SkyTrie {
    subtrie_at(trie, [KEY_EAVT, ein.to_i32()])
}

pub fn ev_iter(
    trie: &SkyTrie,
    a: Attr,
    schema: &Schema,
    order: &HashMap<Ein, u32>,
) -> impl Iterator<Item = (i32, Val)> {
    let aid = schema[a.clone()].ein().to_i32();
    let cardinality = schema[a].cardinality();
    let mut result = Vec::new();
    let evt = subtrie_at(trie, [KEY_AEVT, aid]);
    for (eid, _) in evt.query_all() {
        let mut entries = Vec::new();
        let vt = subtrie_at(trie, [KEY_AEVT, aid, eid]);
        for (vid, _) in vt.query_all() {
            let vt2 = subtrie_at(trie, [KEY_AEVT, aid, eid, vid]);
            for (tx_ein, dir_val) in vt2.query_all() {
                if let TrieValue::U32(dir) = dir_val {
                    entries.push((Vid::from_id(vid), Ein::from(tx_ein), dir_from_u32(dir)));
                }
            }
        }
        for vid in current_vids(entries, cardinality, order) {
            let val = val_table::query(trie, vid)
                .ok()
                .flatten()
                .expect("val not found");
            result.push((eid, val));
        }
    }
    result.into_iter()
}

/// Current facts for a single attribute (AEVT index): every entity whose latest
/// tx on `aid` was an add.
pub fn facts_for_attr(
    trie: &SkyTrie,
    aid: Ein,
    cardinality: Cardinality,
    order: &HashMap<Ein, u32>,
) -> Vec<Fact> {
    let mut result = Vec::new();
    let evt = subtrie_at(trie, [KEY_AEVT, aid.to_i32()]);
    for (eid, _) in evt.query_all() {
        let mut entries = Vec::new();
        let vt = subtrie_at(trie, [KEY_AEVT, aid.to_i32(), eid]);
        for (vid, _) in vt.query_all() {
            let vt2 = subtrie_at(trie, [KEY_AEVT, aid.to_i32(), eid, vid]);
            for (tx_ein, dir_val) in vt2.query_all() {
                if let TrieValue::U32(dir) = dir_val {
                    entries.push((Vid::from_id(vid), Ein::from(tx_ein), dir_from_u32(dir)));
                }
            }
        }
        for vid in current_vids(entries, cardinality, order) {
            let val = val_table::query(trie, vid)
                .ok()
                .flatten()
                .expect("val not found");
            result.push(Fact {
                e: Ein::from(eid),
                a: aid,
                v: val,
            });
        }
    }
    result
}

/// Current facts for a single entity (EAVT index): every attribute it has.
pub fn facts_for_entity(
    trie: &SkyTrie,
    eid: Ein,
    schema: &Schema,
    order: &HashMap<Ein, u32>,
) -> Vec<Fact> {
    let mut result = Vec::new();
    for aid in list_entity_attributes(trie, eid) {
        let card = schema.cardinality_of(aid);
        for (attr_ein, vid) in list_entity_fills(trie, eid, aid, card, order) {
            let val = val_table::query(trie, vid)
                .ok()
                .flatten()
                .expect("val not found");
            result.push(Fact {
                e: eid,
                a: attr_ein,
                v: val,
            });
        }
    }
    result
}

/// All current facts (full EAVT scan).
pub fn all_facts(trie: &SkyTrie, schema: &Schema, order: &HashMap<Ein, u32>) -> Vec<Fact> {
    let mut result = Vec::new();
    for eid in list_entities(trie) {
        result.extend(facts_for_entity(trie, eid, schema, order));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trie::SkyTrie;
    use crate::val;

    #[test]
    fn history_is_retained() {
        let mut trie = SkyTrie::new();
        let tx1 = Ein::tx(&[1u8; 16]);
        let tx2 = Ein::tx(&[2u8; 16]);
        let mut order = HashMap::new();
        order.insert(tx1, 0);
        order.insert(tx2, 1);
        trie = with_update(&trie, Ein(100), Ein(200), val(100), Dir::In, tx1).unwrap();
        trie = with_update(&trie, Ein(100), Ein(200), val(101), Dir::In, tx2).unwrap();

        let fills = list_entity_fills(&trie, Ein(200), Ein(100), Cardinality::One, &order);
        assert_eq!(fills.len(), 1);

        let vt = subtrie_at(&trie, [KEY_EAVT, 200, 100]);
        assert_eq!(vt.query_all().len(), 2);
    }
}
