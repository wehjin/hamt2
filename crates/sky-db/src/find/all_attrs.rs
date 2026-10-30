use std::future::Future;

use crate::find::datalog::binding::{RawPattern, Row};
use crate::traits::Find;
use crate::{Attr, Pod};

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

    fn where_(&self) -> Vec<RawPattern> {
        unreachable!()
    }

    fn process(self, _pod: &Pod, _rows: Vec<Row>) -> Vec<Attr> {
        unreachable!()
    }

    // Attributes live in the schema (not yet materialized as `db/ident` facts),
    // so this is a direct schema query rather than a datalog body.
    fn apply(self, pod: &Pod) -> impl Future<Output = Vec<Attr>>
    where
        Self: Sized,
    {
        async move { pod.schema().to_attrs() }
    }
}
