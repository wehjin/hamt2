use crate::_internal::{db_trie, val_table};
use crate::trie::SkyTrie;
use crate::{Attr, Attribute, Ein, Fill, Schema, Val};
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
}

/// Find helpers
impl Pod {
    pub(crate) fn list_entities(&self) -> Vec<Ein> {
        db_trie::list_entities(&self.trie)
    }

    pub(crate) fn list_attr_eins(&self, ein: Ein) -> Vec<Ein> {
        db_trie::list_entity_attributes(&self.trie, ein)
            .into_iter()
            .map(|attr_ein| attr_ein.ein())
            .collect::<Vec<_>>()
    }

    pub(crate) fn list_fills(&self, ein: Ein) -> Vec<Fill> {
        let mut fills = Vec::new();
        for attr_ein in db_trie::list_entity_attributes(&self.trie, ein) {
            let attr = self
                .schema
                .find_attr(attr_ein.ein())
                .cloned()
                .expect("attr should exist for attr-ein");
            let cardinality = self.schema[attr.clone()].cardinality();
            for (_attr_ein, vid) in db_trie::list_entity_fills(
                &self.trie,
                ein,
                attr_ein,
                cardinality,
                &self.order,
            ) {
                let val = val_table::query(&self.trie, vid)
                    .expect("table should find val")
                    .expect("val should exist");
                fills.push(Fill(attr.clone(), val));
            }
        }
        fills
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
