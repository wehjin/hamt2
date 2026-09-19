mod errors;
mod services;
mod traits;
mod types;

pub use errors::*;
pub use services::*;
pub use traits::*;
pub use types::*;

pub fn query() -> Attr {
    Attr::from("db/query")
}
pub fn ident() -> Attr {
    Attr::from("db/ident")
}
pub fn cardinality() -> Attr {
    Attr::from("db/cardinality")
}
