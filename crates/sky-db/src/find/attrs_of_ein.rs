use crate::find::datalog::atom::Atom;
use crate::traits::Find;
use crate::{Attr, Ein, FindResult, Pod};
use std::future::Future;

pub struct AttrsOfEin(pub Ein);

impl AttrsOfEin {
    pub fn new(ein: impl Into<Ein>) -> Self {
        Self(ein.into())
    }
}

impl Find for AttrsOfEin {
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
        async move {
            // For now, use custom function `list_attr_eins`. Later maybe make a program
            // where the ein is the relator instead of attr.
            pod.list_attr_eins(self.0)
                .into_iter()
                .filter_map(|ein| pod.schema().find_attr(ein).cloned())
                .collect::<Vec<_>>()
        }
    }
}
