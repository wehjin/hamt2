use crate::db::datalog::atom::{Atom, atom};
use crate::db::datalog::term::term;
use crate::db::datalog::var::var;
use crate::db::{Ein, Val, val};
use crate::find::Find;
use crate::find::find_result::FindResult;
use crate::types::Attr;

pub struct ValsInSlot {
    ein: Ein,
    attr: Attr,
}

impl ValsInSlot {
    pub fn new(ein: impl Into<Ein>, attr: Attr) -> Self {
        let ein = ein.into();
        Self { ein, attr }
    }
}

impl Find for ValsInSlot {
    type Output = Val;

    fn select(&self) -> Vec<&'static str> {
        vec!["val"]
    }

    fn where_(&self) -> Vec<Atom> {
        vec![atom(self.attr, [term(val(self.ein)), term(var("val"))])]
    }

    fn process(self, result: FindResult) -> Vec<Self::Output> {
        result
            .into_iter()
            .map(|map| map["val"].clone())
            .collect::<Vec<_>>()
    }
}
