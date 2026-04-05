use crate::db::core::datom::attr::Attr;
use crate::db::find::program::atom::{atom, Atom};
use crate::db::find::program::term::term;
use crate::db::find::program::var::var;
use crate::db::find::Find;
use crate::db::find_result::FindResult;
use crate::db::{val, Ein, Val};

pub struct EinAttrAny {
    ein: Ein,
    attr: Attr,
}

impl EinAttrAny {
    pub fn new(ein: Ein, attr: Attr) -> Self {
        Self { ein, attr }
    }
}

impl Find for EinAttrAny {
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
