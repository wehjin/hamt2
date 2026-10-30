use crate::{AttrEin, Ein, Val};

/// A ground stored fact: an entity, one of its attributes, and a value.
/// This is a row of the universal `holds(e, a, v)` relation.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Fact {
    pub e: Ein,
    pub a: AttrEin,
    pub v: Val,
}
