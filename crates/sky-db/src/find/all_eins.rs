use std::collections::HashSet;

use crate::find::datalog::binding::{RawPattern, Row, pattern};
use crate::find::datalog::var::var;
use crate::traits::Find;
use crate::{Ein, Pod};

pub struct AllEins;

impl AllEins {
    pub fn new() -> Self {
        Self
    }
}

impl Find for AllEins {
    type Output = Ein;

    fn select(&self) -> Vec<&'static str> {
        vec!["ein"]
    }

    fn where_(&self) -> Vec<RawPattern> {
        vec![pattern(var("ein"), var("a"), var("v"))]
    }

    fn process(self, _pod: &Pod, rows: Vec<Row>) -> Vec<Ein> {
        rows.into_iter()
            .filter_map(|r| r["ein"].as_ein())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }
}
