use crate::db::Attr;

pub fn query() -> Attr {
    Attr::from("db/query")
}
pub fn ident() -> Attr {
    Attr::from("db/ident")
}
pub fn cardinality() -> Attr {
    Attr::from("db/cardinality")
}
