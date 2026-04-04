use std::fmt::Display;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Attr(pub &'static str, pub &'static str);

impl Attr {
    pub const DB_IDENT: Attr = Attr("db", "ident");
    pub fn to_ident(&self) -> String {
        format!("{}/{}", self.0, self.1)
    }
}

impl Display for Attr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_ident().as_str())
    }
}
