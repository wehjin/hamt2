use crate::crate_services::datalog::atom::Atom;
use crate::db::{Db, db_trie};
use crate::find::ValsInSlot;
use futures::FutureExt;
use sky_types::db::schema::Schema;
use sky_types::db::{Attr, Ein, FindResult, QueryError, Val};
use sky_types::storage::BaseStore;
use sky_types::trie::TrieStream;

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

impl<S: BaseStore + Send + Sync> DbQuery for Db<S> {
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
        T: TrieStream,
    {
        async move {
            let select = self.select();
            let where_ = self.where_();
            let result = db_trie::find(trie, schema, select, where_).await;
            self.process(result)
        }
    }
}
