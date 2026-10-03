use crate::_internal::datalog::atom::{Atom, atom};
use crate::_internal::datalog::term::term;
use crate::_internal::datalog::var::var;
use crate::Attr;
use crate::FindResult;
use crate::traits::Find;
use crate::{Ein, Val, val};

pub struct ValsInSlot(pub Ein, pub Attr);

impl ValsInSlot {
    pub fn new(ein: impl Into<Ein>, attr: impl Into<Attr>) -> Self {
        Self(ein.into(), attr.into())
    }
}

impl Find for ValsInSlot {
    type Output = Val;

    fn select(&self) -> Vec<&'static str> {
        vec!["val"]
    }

    fn where_(&self) -> Vec<Atom> {
        vec![atom(self.1.clone(), [term(val(self.0)), term(var("val"))])]
    }

    fn process(self, result: FindResult) -> Vec<Self::Output> {
        result
            .into_iter()
            .map(|map| map["val"].clone())
            .collect::<Vec<_>>()
    }
}
