use crate::find::datalog::binding::{RawPattern, Row, pattern};
use crate::find::datalog::var::var;
use crate::traits::Find;
use crate::{Attr, Ein, Pod, Val};

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

    fn where_(&self) -> Vec<RawPattern> {
        vec![pattern(var("ein"), self.attr.clone(), var("val"))]
    }

    fn process(self, _pod: &Pod, rows: Vec<Row>) -> Vec<(Ein, Val)> {
        rows.into_iter()
            .map(|r| {
                (
                    r["ein"].as_ein().expect("ein column"),
                    r["val"].to_val(),
                )
            })
            .collect()
    }
}
