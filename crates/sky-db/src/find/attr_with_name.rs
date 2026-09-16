use crate::crate_services::datalog::atom::Atom;
use crate::db::Schema;
use crate::find::Find;
use crate::trie::prelude::*;
use sky_types::db::{Attr, FindResult};
use std::future::Future;

pub struct AttrWithName {
    attr: Attr,
}

impl AttrWithName {
    pub fn new(attr: Attr) -> Self {
        Self { attr }
    }
}

impl Find for AttrWithName {
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

    fn apply<T>(self, _trie: &T, schema: &Schema) -> impl Future<Output = Vec<Self::Output>>
    where
        Self: Sized,
        T: TrieQuery,
    {
        async move {
            if schema.contains(&self.attr) {
                vec![self.attr]
            } else {
                vec![]
            }
        }
    }
}
