use crate::db::{Datom, TransactError};

#[allow(async_fn_in_trait)]
pub trait Transact {
    async fn transact(&mut self, datoms: impl Into<Vec<Datom>>) -> Result<&mut Self, TransactError>
    where
        Self: Sized;
}
