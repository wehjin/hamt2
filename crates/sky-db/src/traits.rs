use crate::TransactError;
use crate::find::ValsInSlot;
use crate::find::datalog::binding::{RawPattern, Row};
use crate::{Attr, Ein, QueryError, Val};
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
    fn where_(&self) -> Vec<RawPattern>;
    fn process(self, pod: &Pod, rows: Vec<Row>) -> Vec<Self::Output>;

    fn apply(self, pod: &Pod) -> impl Future<Output = Vec<Self::Output>>
    where
        Self: Sized,
    {
        async move {
            let rows = crate::find::run::run_find(pod, self.select(), self.where_());
            self.process(pod, rows)
        }
    }
}
