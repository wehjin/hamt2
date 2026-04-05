use crate::db::core::datom::attr::Attr;
use crate::db::find::program::atom::{atom, Atom};
use crate::db::find::program::term::term;
use crate::db::find::program::var::var;
use crate::db::find::Find;
use crate::db::find_result::FindResult;
use crate::db::{Ein, Val};

pub struct AnyAttrAny {
    attr: Attr,
}

impl AnyAttrAny {
    pub fn new(attr: Attr) -> Self {
        Self { attr }
    }
}

impl Find for AnyAttrAny {
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
