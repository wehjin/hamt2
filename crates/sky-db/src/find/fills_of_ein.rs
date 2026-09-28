use crate::crate_services::datalog::atom::Atom;
use crate::crate_services::val_table;
use crate::db::db_trie;
use crate::traits::Find;
use sky_types::db::schema::Schema;
use sky_types::db::{Ein, Fill, FindResult};
use sky_types::trie::TrieWalk;

pub struct FillsOfEin(pub Ein);

impl FillsOfEin {
    pub fn new(ein: impl Into<Ein>) -> Self {
        Self(ein.into())
    }
}

impl Find for FillsOfEin {
    type Output = Fill;

    fn select(&self) -> Vec<&'static str> {
        unreachable!()
    }

    fn where_(&self) -> Vec<Atom> {
        unreachable!()
    }

    fn process(self, _result: FindResult) -> Vec<Self::Output> {
        unreachable!()
    }

    fn apply<T>(self, trie: &T, schema: &Schema) -> impl Future<Output = Vec<Self::Output>>
    where
        Self: Sized,
        T: TrieWalk,
    {
        async move {
            // For now, use custom function `list_entity_attributes`. Later maybe make a program
            // where the ein is the relator instead of attr.
            let mut fills = Vec::new();
            let mut pre_fills = Vec::new();
            for attr_ein in db_trie::list_entity_attributes(trie, self.0).await {
                let attr_fills = db_trie::list_entity_fills(trie, self.0, attr_ein).await;
                pre_fills.extend(attr_fills);
            }
            for (attr_ein, vid) in pre_fills {
                let attr = schema
                    .find_attr(attr_ein.ein())
                    .cloned()
                    .expect("attr should exist for attr-ein");
                let val = val_table::query(trie, vid)
                    .await
                    .expect("table should find val")
                    .expect("val should exist");
                fills.push(Fill(attr, val))
            }
            fills
        }
    }
}
