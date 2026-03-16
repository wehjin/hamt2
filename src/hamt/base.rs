#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Txid(usize);

impl Txid {
    pub const FLOOR: Txid = Txid(0);
}

pub struct Attr(pub u32);

pub struct Ent(pub u32);
