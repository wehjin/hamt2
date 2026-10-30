use crate::find::datalog::binding::{RawPattern, Row, pattern};
use crate::find::datalog::var::var;
use crate::traits::Find;
use crate::{Attr, Ein, Pod};

pub struct AttrsOfEin(pub Ein);

impl AttrsOfEin {
    pub fn new(ein: impl Into<Ein>) -> Self {
        Self(ein.into())
    }
}

impl Find for AttrsOfEin {
    type Output = Attr;

    fn select(&self) -> Vec<&'static str> {
        vec!["a"]
    }

    fn where_(&self) -> Vec<RawPattern> {
        vec![pattern(self.0, var("a"), var("v"))]
    }

    fn process(self, pod: &Pod, rows: Vec<Row>) -> Vec<Attr> {
        let mut attrs = rows
            .into_iter()
            .filter_map(|r| {
                r["a"]
                    .as_ein()
                    .and_then(|e| pod.schema().find_attr(e).cloned())
            })
            .collect::<Vec<_>>();
        attrs.sort();
        attrs.dedup();
        attrs
    }
}
