use crate::TransactError;
use crate::find::ValsInSlot;
use crate::find::datalog::atom::Atom;
use crate::{Attr, Ein, FindResult, QueryError, Val};
use crate::{Datom, Pod};
use futures::FutureExt;

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

impl DbQuery for Pod {
    async fn find<F: Find>(&self, find: F) -> Vec<F::Output> {
        find.apply(self).await
    }
}

pub trait Find {
    type Output;

    fn select(&self) -> Vec<&'static str>;
    fn where_(&self) -> Vec<Atom>;
    fn process(self, result: FindResult) -> Vec<Self::Output>;

    fn apply(self, pod: &Pod) -> impl Future<Output = Vec<Self::Output>>
    where
        Self: Sized,
    {
        async move {
            let select = self.select();
            let where_ = self.where_();
            let result = crate::find::run::run_find(pod, select, where_);
            self.process(result)
        }
    }
}
