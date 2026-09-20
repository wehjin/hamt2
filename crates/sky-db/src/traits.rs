use crate::crate_services::datalog::atom::Atom;
use crate::db::{Db, db_trie};
use crate::find::ValsInSlot;
use futures::FutureExt;
use sky_types::db::schema::Schema;
use sky_types::db::{Attr, Ein, FindResult, QueryError, Val};
use sky_types::storage::ReadWriteStorage;
use sky_types::trie::{SlotBaseId, TrieQuery};

pub trait DbQuery {
    fn find<F: Find>(&self, find: F) -> impl Future<Output = Vec<F::Output>>;

    fn find_val(
        &self,
        e: impl Into<Ein>,
        a: Attr,
    ) -> impl Future<Output = Result<Option<Val>, QueryError>> {
        async move {
            let find = self.find(ValsInSlot::new(e, a)).await;
            Ok(find.first().cloned())
        }
    }

    fn get_val(&self, e: impl Into<Ein>, a: Attr) -> impl Future<Output = Val> {
        self.find_val(e.into(), a).map(|v| {
            v.expect("find_val should succeed")
                .expect("value should exist")
        })
    }
}

impl<S: ReadWriteStorage> DbQuery for Db<S> {
    fn find<F: Find>(&self, find: F) -> impl Future<Output = Vec<F::Output>> {
        find.apply(&self.trie, &self.schema)
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
        T: TrieQuery<SlotBaseId>,
    {
        async move {
            let select = self.select();
            let where_ = self.where_();
            let result = db_trie::find(trie, schema, select, where_).await;
            self.process(result)
        }
    }
}
