use crate::QueryError;
use crate::db::Schema;
use crate::db::db_trie;
use crate::db::{Attr, Ein};
use crate::find::Find;
use crate::find::find_result::FindResult;
use crate::crate_services::datalog::atom::Atom;
use crate::trie::prelude::*;
use std::future::Future;

pub struct AttrsOfEin {
    ein: Ein,
}

impl AttrsOfEin {
    pub fn new(ein: impl Into<Ein>) -> Self {
        let ein = ein.into();
        Self { ein }
    }
}

impl Find for AttrsOfEin {
    type Output = Attr;

    fn select(&self) -> Vec<&'static str> {
        unreachable!()
    }

    fn where_(&self) -> Vec<Atom> {
        unreachable!()
    }

    fn process(self, _result: FindResult) -> Vec<Self::Output> {
        unreachable!()
    }

    fn apply<T, S>(
        self,
        trie: &T,
        schema: &Schema,
    ) -> impl Future<Output = Result<Vec<Self::Output>, QueryError>>
    where
        Self: Sized,
        T: TrieQuery<S>,
        S: ReadTrieStorage,
    {
        async move {
            // For now, use custom function `list_entity_attributes`. Later maybe make a program
            // where the ein is the relator instead of attr.
            let attr_eins = db_trie::list_entity_attributes(trie, self.ein).await;
            let attrs = attr_eins
                .into_iter()
                .filter_map(|attr_ein| schema.find_attr(attr_ein).copied())
                .collect::<Vec<_>>();
            Ok(attrs)
        }
    }
}
