use crate::crate_services::datalog::atom::Atom;
use crate::db::Schema;
use crate::traits::Find;
use sky_types::db::Attr;
use sky_types::db::FindResult;
use sky_types::trie::*;
use std::future::Future;

pub struct AllAttrs;

impl AllAttrs {
    pub fn new() -> Self {
        Self
    }
}

impl Find for AllAttrs {
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
        T: TrieWalk,
    {
        async move { schema.to_attrs() }
    }
}
