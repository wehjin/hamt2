use crate::db::core::datom::attr::Attr;
use crate::db::find::program::atom::{atom, Atom};
use crate::db::find::program::term::term;
use crate::db::find::program::var::var;
use crate::db::find::Find;
use crate::db::find_result::FindResult;
use crate::db::Ein;

pub struct AnyAttrIgnore {
    attr: Attr,
}

impl AnyAttrIgnore {
    pub fn new(attr: Attr) -> Self {
        Self { attr }
    }
}

impl Find for AnyAttrIgnore {
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
