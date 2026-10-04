use crate::{Ein, Entity, Val};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Bind(pub Entity, pub Val);

impl Bind {
    pub fn new(entity: Entity, val: impl Into<Val>) -> Self {
        Self(entity, val.into())
    }
    pub fn ein(&self) -> &Ein {
        self.0.ein()
    }
    pub fn val(&self) -> &Val {
        &self.1
    }
}
