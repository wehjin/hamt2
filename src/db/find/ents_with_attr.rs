use crate::db::find::Rule;
use crate::db::key::KEY_AEVT;
use crate::db::{Attr, Ent};
use crate::hamt::trie::space::SpaceTrie;
use crate::space::Space;
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
    fn update<T: Space>(
        &mut self,
        trie: &SpaceTrie<T>,
        attrs: &HashMap<Attr, Ent>,
    ) -> Result<bool, QueryError> {
        let aid = attrs.get(&self.attr).expect("attr should exist").to_id();
        let aevt_key = [KEY_AEVT, aid];
        let eids = if let Some(value) = trie.deep_query_value(aevt_key)? {
            let subtrie = trie.to_subtrie_from_value(value)?;
            let key_values = subtrie.query_key_values()?;
            dbg!("rule-time");
            dbg!(&key_values);
            let eids = key_values
                .into_iter()
                .map(|(eid, _)| Ent(eid))
                .collect::<Vec<_>>();
            eids
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
