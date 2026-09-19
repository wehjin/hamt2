use crate::db::Db;
use crate::find::{Find, ValsInSlot};
use futures::FutureExt;
use sky_trie::prelude::ReadWriteStorage;
use sky_types::db::{Attr, Ein, QueryError, Val};

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