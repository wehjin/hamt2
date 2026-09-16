use crate::crate_services::datalog::atom::{Atom, atom};
use crate::crate_services::datalog::term::term;
use crate::crate_services::datalog::var::var;
use crate::find::Find;
use sky_types::db::{Attr, Ein, FindResult, Val};

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
        vec![atom(self.attr.clone(), [term(var("ein")), term(var("val"))])]
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
