use crate::Ein;
use crate::FindResult;
use crate::find::datalog::atom::Atom;
use crate::traits::Find;
use crate::Pod;
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

    fn apply(self, pod: &Pod) -> impl Future<Output = Vec<Self::Output>>
    where
        Self: Sized,
    {
        async move { pod.list_entities() }
    }
}
