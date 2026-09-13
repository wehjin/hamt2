use crate::db::Attr;
use crate::db::datalog::atom::{Atom, atom};
use crate::db::datalog::term::term;
use crate::db::datalog::var::var;
use crate::db::{Ein, Val};
use crate::find::Find;
use crate::find::find_result::FindResult;

pub struct BindsForAttr {
    attr: Attr,
}

impl BindsForAttr {
    pub fn new(attr: Attr) -> Self {
        Self { attr }
    }
}

impl Find for BindsForAttr {
    type Output = (Ein, Val);

    fn select(&self) -> Vec<&'static str> {
        vec!["ein", "val"]
    }

    fn where_(&self) -> Vec<Atom> {
        vec![atom(self.attr, [term(var("ein")), term(var("val"))])]
    }

    fn process(self, result: FindResult) -> Vec<Self::Output> {
        result
            .into_iter()
            .map(|map| {
                let ein = Ein::from(map["ein"].clone());
                let val = Val::from(map["val"].clone());
                (ein, val)
            })
            .collect::<Vec<_>>()
    }
}
