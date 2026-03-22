use crate::db::find::Rule;
use crate::db::{Attr, Ent, KEY_AEVT};
use crate::hamt::trie::mem::value::MemValue;
use crate::hamt::trie::space::SpaceTrie;
use crate::QueryError;
use std::collections::HashMap;

pub struct EntsWithAttr {
    name: &'static str,
    attr: Attr,
    values: Vec<Ent>,
}

impl EntsWithAttr {
    pub fn new(name: &'static str, attr: Attr) -> Self {
        Self {
            name,
            attr,
            values: Vec::new(),
        }
    }
}

impl Rule for EntsWithAttr {
    type Output = Ent;

    fn results(&self, name: &'static str) -> &[Self::Output] {
        if name == self.name { &self.values } else { &[] }
    }
    fn update(
        &mut self,
        trie: &SpaceTrie,
        attrs: &HashMap<Attr, Ent>,
    ) -> Result<bool, QueryError> {
        let aid = attrs.get(&self.attr).expect("attr should exist").to_id();
        let aevt_key = [KEY_AEVT, aid];
        let eids = if let Some(MemValue::MapBase(map_base)) = trie.deep_query_value(aevt_key)? {
            trie.to_subtrie(map_base)
                .query_key_values()?
                .into_iter()
                .map(|(eid, _)| Ent(eid))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        if eids == self.values {
            Ok(false)
        } else {
            self.values = eids;
            Ok(true)
        }
    }
}
