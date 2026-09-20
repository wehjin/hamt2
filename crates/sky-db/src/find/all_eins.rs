use crate::crate_services::datalog::atom::Atom;
use crate::db::Schema;
use crate::db::db_trie;
use crate::traits::Find;
use crate::trie::prelude::*;
use sky_types::db::Ein;
use sky_types::db::FindResult;
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
        T: TrieQuery<SlotBaseId>,
    {
        db_trie::list_entities(trie)
    }
}
