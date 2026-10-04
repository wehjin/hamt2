use crate::Attr;
use crate::FindResult;
use crate::find::datalog::atom::Atom;
use crate::traits::Find;
use crate::Pod;
use std::future::Future;

pub struct AllAttrs;

impl AllAttrs {
    pub fn new() -> Self {
        Self
    }
}

impl Find for AllAttrs {
    type Output = Attr;

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
        async move { pod.schema().to_attrs() }
    }
}
