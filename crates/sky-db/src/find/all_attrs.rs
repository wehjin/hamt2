use crate::find::datalog::binding::{RawPattern, Row, pattern};
use crate::find::datalog::var::var;
use crate::traits::Find;
use crate::{Attr, Pod, db};

pub struct AllAttrs;

impl AllAttrs {
    pub fn new() -> Self {
        Self
    }
}

impl Find for AllAttrs {
    type Output = Attr;

    fn select(&self) -> Vec<&'static str> {
        vec!["a"]
    }

    fn where_(&self) -> Vec<RawPattern> {
        vec![pattern(var("a"), db::ident(), var("n"))]
    }

    fn process(self, pod: &Pod, rows: Vec<Row>) -> Vec<Attr> {
        rows.into_iter()
            .filter_map(|r| r["a"].as_ein().and_then(|e| pod.schema().find_attr(e).cloned()))
            .collect()
    }
}
