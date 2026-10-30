use crate::find::datalog::binding::{RawPattern, Row, pattern};
use crate::find::datalog::var::var;
use crate::traits::Find;
use crate::{Ein, Fill, Pod};

pub struct EntityFills(pub Ein);

impl EntityFills {
    pub fn new(ein: impl Into<Ein>) -> Self {
        Self(ein.into())
    }
}

impl Find for EntityFills {
    type Output = Fill;

    fn select(&self) -> Vec<&'static str> {
        vec!["a", "v"]
    }

    fn where_(&self) -> Vec<RawPattern> {
        vec![pattern(self.0, var("a"), var("v"))]
    }

    fn process(self, pod: &Pod, rows: Vec<Row>) -> Vec<Fill> {
        rows.into_iter()
            .map(|r| {
                let aid = r["a"].as_attr().expect("attr column");
                let attr = pod
                    .schema()
                    .find_attr(aid.ein())
                    .cloned()
                    .expect("attr ident");
                Fill(attr, r["v"].to_val())
            })
            .collect()
    }
}
