use crate::find::datalog::binding::{RawPattern, Row, pattern};
use crate::find::datalog::var::var;
use crate::traits::Find;
use crate::{Attr, Ein, Pod, Val};

pub struct ValsInSlot(pub Ein, pub Attr);

impl ValsInSlot {
    pub fn new(ein: impl Into<Ein>, attr: impl Into<Attr>) -> Self {
        Self(ein.into(), attr.into())
    }
}

impl Find for ValsInSlot {
    type Output = Val;

    fn select(&self) -> Vec<&'static str> {
        vec!["val"]
    }

    fn where_(&self) -> Vec<RawPattern> {
        vec![pattern(self.0, self.1.clone(), var("val"))]
    }

    fn process(self, _pod: &Pod, rows: Vec<Row>) -> Vec<Val> {
        rows.into_iter().map(|r| r["val"].to_val()).collect()
    }
}
