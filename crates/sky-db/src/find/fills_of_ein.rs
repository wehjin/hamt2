use crate::find::datalog::atom::Atom;
use crate::traits::Find;
use crate::{Ein, Fill, FindResult, Pod};
use std::future::Future;

pub struct EntityFills(pub Ein);

impl EntityFills {
    pub fn new(ein: impl Into<Ein>) -> Self {
        Self(ein.into())
    }
}

impl Find for EntityFills {
    type Output = Fill;

    fn select(&self) -> Vec<&'static str> {
        unreachable!()
    }

    fn where_(&self) -> Vec<Atom> {
        unreachable!()
    }

    fn process(self, _result: FindResult) -> Vec<Self::Output> {
        unreachable!()
    }

    fn apply(self, pod: &Pod) -> impl Future<Output = Vec<Self::Output>>
    where
        Self: Sized,
    {
        async move { pod.list_fills(self.0) }
    }
}
