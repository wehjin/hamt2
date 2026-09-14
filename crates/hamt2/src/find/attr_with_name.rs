use crate::crate_services::datalog::atom::Atom;
use crate::db::AttrName;
use crate::db::{Attr, Schema};
use crate::find::Find;
use crate::trie::prelude::*;
use sky_types::FindResult;
use std::future::Future;

pub struct AttrWithName {
    attr_name: AttrName,
}

impl AttrWithName {
    pub fn new(attr_name: AttrName) -> Self {
        Self { attr_name }
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

    fn apply<T, S>(self, _trie: &T, schema: &Schema) -> impl Future<Output = Vec<Self::Output>>
    where
        Self: Sized,
        T: StorageTrieQuery<S>,
        S: ReadTrieStorage,
    {
        async move {
            let attr = schema.find_attr_by_name(&self.attr_name);
            attr.into_iter().cloned().collect::<Vec<_>>()
        }
    }
}
