use std::fmt::Display;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Attr(pub &'static str);

impl Attr {
    pub fn as_ident(&self) -> &'static str {
        self.0
    }
}

impl Display for Attr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ident())
    }
}
