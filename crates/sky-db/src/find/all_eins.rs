use crate::Ein;
use crate::FindResult;
use crate::Schema;
use crate::_internal::datalog::atom::Atom;
use crate::_internal::db_trie;
use crate::traits::Find;
use crate::trie::*;
use std::future::Future;

pub struct AllEins;

impl AllEins {
    pub fn new() -> Self {
        Self
    }
}

impl Find for AllEins {
    type Output = Ein;

    fn select(&self) -> Vec<&'static str> {
        unreachable!()
    }

    fn where_(&self) -> Vec<Atom> {
        unreachable!()
    }

    fn process(self, _result: FindResult) -> Vec<Self::Output> {
        unreachable!()
    }

    fn apply<T>(self, trie: &T, _schema: &Schema) -> impl Future<Output = Vec<Self::Output>>
    where
        Self: Sized,
        T: QueryCursor + KvStream + Snap,
    {
        async move { db_trie::list_entities(trie) }
    }
}
