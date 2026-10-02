use crate::Schema;
use crate::TransactError;
use crate::find::ValsInSlot;
use crate::_internal::datalog::atom::Atom;
use crate::_internal::db_trie;
use crate::{Attr, Ein, FindResult, QueryError, Val};
use crate::{Datom, Db};
use futures::FutureExt;
use sky_types::trie::{QueryCursor, TrieQuery, TrieSnap, TrieStream};

#[allow(async_fn_in_trait)]
pub trait Transact {
    async fn transact(&mut self, datoms: impl Into<Vec<Datom>>) -> Result<&mut Self, TransactError>
    where
        Self: Sized;
}

#[allow(async_fn_in_trait)]
pub trait DbQuery {
    async fn find<F: Find>(&self, find: F) -> Vec<F::Output>;

    async fn find_val(&self, e: impl Into<Ein>, a: Attr) -> Result<Option<Val>, QueryError> {
        let find = self.find(ValsInSlot::new(e, a)).await;
        Ok(find.first().cloned())
    }

    async fn get_val(&self, e: impl Into<Ein>, a: Attr) -> Val {
        self.find_val(e.into(), a)
            .map(|v| {
                v.expect("find_val should succeed")
                    .expect("value should exist")
            })
            .await
    }
}

impl DbQuery for Db {
    async fn find<F: Find>(&self, find: F) -> Vec<F::Output> {
        find.apply(&self.trie, &self.schema).await
    }
}

pub trait Find {
    type Output;

    fn select(&self) -> Vec<&'static str>;
    fn where_(&self) -> Vec<Atom>;
    fn process(self, result: FindResult) -> Vec<Self::Output>;

    fn apply<T>(self, trie: &T, schema: &Schema) -> impl Future<Output = Vec<Self::Output>>
    where
        Self: Sized,
        T: QueryCursor + TrieStream + TrieSnap + TrieQuery,
    {
        async move {
            let select = self.select();
            let where_ = self.where_();
            let result = db_trie::find(trie, schema, select, where_).await;
            self.process(result)
        }
    }
}
