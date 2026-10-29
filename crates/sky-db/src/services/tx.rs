use crate::Attr;

pub fn author() -> Attr {
    Attr::from("tx/author")
}
pub fn prev() -> Attr {
    Attr::from("tx/prev")
}
pub fn nonce() -> Attr {
    Attr::from("tx/nonce")
}
pub fn instant() -> Attr {
    Attr::from("tx/instant")
}
