use crate::db::component::val_table;
use crate::db::find::Rule;
use crate::db::key::KEY_EAVT;
use crate::db::vid::Vid;
use crate::db::{Attr, Ent, Val};
use crate::hamt::trie::space::trie::SpaceTrie;
use crate::space::Space;
use crate::QueryError;
use std::collections::HashMap;

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

    fn update<T: Space>(
        &mut self,
        trie: &SpaceTrie<T>,
        attrs: &HashMap<Attr, Ent>,
    ) -> Result<bool, QueryError> {
        let eid = self.ent.to_id();
        let aid = attrs.get(&self.attr).expect("attr should exist").to_id();
        let eavt_key = [KEY_EAVT, eid, aid];
        let eavt_value = trie.deep_query_value(eavt_key)?;
        if let Some(mem_value) = eavt_value {
            let subtrie = trie.to_subtrie_from_value(mem_value)?;
            let key_values = subtrie.query_keys_values()?;
            let first_key_value = key_values.first();
            if let Some((vid, _)) = first_key_value {
                let val = val_table::query(&trie, Vid::from_id(*vid))?.expect("val should exist");
                self.vals.push(val);
            }
        }
        Ok(true)
    }
}
