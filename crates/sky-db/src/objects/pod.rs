use crate::_internal::db_trie;
use crate::trie::SkyTrie;
use crate::{Attr, AttrEin, Attribute, Ein, Fact, Schema, Val};
use std::collections::HashMap;

/// A stable, immutable snapshot of a peer's index. Query-only.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Pod {
    pub(crate) schema: Schema,
    pub(crate) trie: SkyTrie,
    pub(crate) order: HashMap<Ein, u32>,
}

/// Queries
impl Pod {
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn ev_iter(&self, a: Attr) -> impl Iterator<Item = (i32, Val)> {
        db_trie::ev_iter(&self.trie, a, &self.schema, &self.order)
    }

    pub fn facts_for_attr(&self, aid: AttrEin) -> Vec<Fact> {
        let card = self.schema.cardinality_of(aid);
        db_trie::facts_for_attr(&self.trie, aid, card, &self.order)
    }

    pub fn facts_for_entity(&self, eid: Ein) -> Vec<Fact> {
        db_trie::facts_for_entity(&self.trie, eid, &self.schema, &self.order)
    }

    pub fn all_facts(&self) -> Vec<Fact> {
        db_trie::all_facts(&self.trie, &self.schema, &self.order)
    }
}

/// Attributes
impl Pod {
    pub fn get_attribute(&self, attr: impl Into<Attr>) -> Option<Attribute> {
        let details = self.schema.get_details(attr);
        details.map(|details| Attribute::new(details, self.clone()))
    }
    pub async fn list_attributes(&self) -> Vec<Attribute> {
        self.schema
            .list_details()
            .into_iter()
            .map(|details| Attribute::new(details, self.clone()))
            .collect::<Vec<_>>()
    }
}
