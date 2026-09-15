use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Attr(pub &'static str);

impl Attr {
    pub fn as_ident(&self) -> &'static str {
        self.0
    }
    pub fn to_name(&self) -> String {
        self.as_ident().to_string()
    }
}

impl From<&'static str> for Attr {
    fn from(value: &'static str) -> Self {
        Self(value)
    }
}

impl Display for Attr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ident())
    }
}
