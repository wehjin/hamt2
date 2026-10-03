use crate::_internal::datalog::atom::Atom;
use crate::_internal::db_trie;
use crate::Schema;
use crate::traits::Find;
use crate::trie::*;
use crate::{Attr, Ein, FindResult};
use std::future::Future;

pub struct AttrsOfEin(pub Ein);

impl AttrsOfEin {
    pub fn new(ein: impl Into<Ein>) -> Self {
        Self(ein.into())
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

    fn apply<T>(self, trie: &T, schema: &Schema) -> impl Future<Output = Vec<Self::Output>>
    where
        Self: Sized,
        T: QueryCursor + KvStream + Snap,
    {
        async move {
            // For now, use custom function `list_entity_attributes`. Later maybe make a program
            // where the ein is the relator instead of attr.
            let attr_eins = db_trie::list_entity_attributes(trie, self.0).await;
            let attrs = attr_eins
                .into_iter()
                .filter_map(|attr_ein| schema.find_attr(attr_ein.ein()).cloned())
                .collect::<Vec<_>>();
            attrs
        }
    }
}
