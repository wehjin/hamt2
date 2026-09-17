use sky_types::db::Attr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("Unknown attribute: {0:?}")]
    UnknownAttr(Attr),
}
