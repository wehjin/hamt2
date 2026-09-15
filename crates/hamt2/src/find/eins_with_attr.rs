use crate::crate_services::datalog::atom::{Atom, atom};
use crate::crate_services::datalog::term::term;
use crate::crate_services::datalog::var::var;
use crate::find::Find;
use sky_types::db::{Attr, Ein, FindResult};

pub struct EinsWithAttr {
    attr: Attr,
}

impl EinsWithAttr {
    pub fn new(attr: Attr) -> Self {
        Self { attr }
    }
}

impl Find for EinsWithAttr {
    type Output = Ein;

    fn select(&self) -> Vec<&'static str> {
        vec!["ein"]
    }

    fn where_(&self) -> Vec<Atom> {
        vec![atom(self.attr, [term(var("ein")), term(var("ignore"))])]
    }

    fn process(self, result: FindResult) -> Vec<Self::Output> {
        result
            .into_iter()
            .map(|map| Ein::from(map["ein"].clone()))
            .collect::<Vec<_>>()
    }
}
