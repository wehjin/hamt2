use crate::find::datalog::binding::{RawPattern, Row, pattern};
use crate::find::datalog::var::var;
use crate::traits::Find;
use crate::{Attr, Ein, Pod};

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

    fn where_(&self) -> Vec<RawPattern> {
        vec![pattern(var("ein"), self.attr.clone(), var("ignore"))]
    }

    fn process(self, _pod: &Pod, rows: Vec<Row>) -> Vec<Ein> {
        rows.into_iter()
            .map(|r| r["ein"].as_ein().expect("ein column"))
            .collect()
    }
}
