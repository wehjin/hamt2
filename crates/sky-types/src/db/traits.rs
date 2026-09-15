use crate::db::Datom;
use crate::db::TransactError;

#[allow(async_fn_in_trait)]
pub trait Transact {
    async fn transact(self, datoms: impl Into<Vec<Datom>>) -> Result<Self, TransactError>
    where
        Self: Sized;
}
