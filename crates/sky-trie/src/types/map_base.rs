use crate::{Base, SlotMap};
use std::sync::{Arc, LazyLock};

static EMPTY_BASE: LazyLock<Arc<Base>> = LazyLock::new(|| Arc::new(Base::empty()));

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MapBase {
    pub map: SlotMap,
    pub base: Arc<Base>,
}

impl MapBase {
    pub fn empty() -> Self {
        Self {
            map: SlotMap::empty(),
            base: EMPTY_BASE.clone(),
        }
    }
}
