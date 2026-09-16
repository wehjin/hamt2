use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Attr(pub String);

impl Attr {
    pub fn as_ident(&self) -> &str {
        &self.0
    }
    pub fn to_name(&self) -> String {
        self.0.clone()
    }
}

impl From<&str> for Attr {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for Attr {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Display for Attr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ident())
    }
}
