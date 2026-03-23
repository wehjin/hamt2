use crate::db::find::Rule;
use crate::db::{Attr, Ent, Val};
use crate::hamt::trie::mem::value::MemValue;
use crate::hamt::trie::space::SpaceTrie;
use crate::QueryError;
use std::collections::HashMap;
use crate::db::key::{KEY_EAVT, KEY_VALS};

pub struct ValsWithEntAttr {
    ent: Ent,
    attr: Attr,
    vals: Vec<Val>,
}

impl ValsWithEntAttr {
    pub fn new(_name: &'static str, ent: Ent, attr: Attr) -> Self {
        Self {
            ent,
            attr,
            vals: Vec::new(),
        }
    }
}

impl Rule for ValsWithEntAttr {
    type Output = Val;

    fn results(&self, _name: &'static str) -> &[Self::Output] {
        &self.vals
    }

    fn update(&mut self, trie: &SpaceTrie, attrs: &HashMap<Attr, Ent>) -> Result<bool, QueryError> {
        let eid = self.ent.to_id();
        let aid = attrs.get(&self.attr).expect("attr should exist").to_id();
        let eavt_key = [KEY_EAVT, eid, aid];
        let eavt_value = trie.deep_query_value(eavt_key)?;
        if let Some(MemValue::MapBase(map_base)) = eavt_value {
            let subtrie = trie.to_subtrie(map_base);
            let key_values = subtrie.query_key_values()?;
            let first_key_value = key_values.first();
            if let Some((vid, _)) = first_key_value {
                let mem_val = trie
                    .deep_query_value([KEY_VALS, *vid])?
                    .expect("mem_val should exist for vid");
                let val = Val::from(mem_val);
                self.vals.push(val);
            }
        }
        Ok(true)
    }
}
