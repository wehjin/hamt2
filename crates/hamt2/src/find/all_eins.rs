use crate::QueryError;
use crate::db::Ein;
use crate::db::Schema;
use crate::db::db_trie;
use crate::find::Find;
use crate::find::find_result::FindResult;
use crate::crate_services::datalog::atom::Atom;
use crate::trie::prelude::*;
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

    fn apply<T, S>(
        self,
        trie: &T,
        _schema: &Schema,
    ) -> impl Future<Output = Result<Vec<Self::Output>, QueryError>>
    where
        Self: Sized,
        T: TrieQuery<S>,
        S: ReadTrieStorage,
    {
        async move {
            let eins = db_trie::list_entities(trie).await;
            Ok(eins)
        }
    }
}
