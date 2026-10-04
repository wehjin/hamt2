use crate::find::{AttrsOfEin, ValsInSlot};
use crate::{Attr, DbQuery, Ein, Pod, Val};
use std::cmp::Ordering;
use std::fmt;

#[derive(Clone, Eq, PartialEq)]
pub struct Entity {
    ein: Ein,
    pod: Pod,
}

impl Entity {
    pub fn new(ein: Ein, pod: Pod) -> Self {
        Self { ein, pod }
    }

    pub fn ein(&self) -> &Ein {
        &self.ein
    }

    pub async fn list_attrs(&self) -> Vec<Attr> {
        self.pod.find(AttrsOfEin(self.ein)).await
    }

    pub async fn get_value(&self, attr: impl Into<Attr>) -> Option<Val> {
        self.pod
            .find(ValsInSlot(self.ein, attr.into()))
            .await
            .first()
            .cloned()
    }
}

impl Ord for Entity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ein.cmp(&other.ein)
    }
}
impl PartialOrd for Entity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.ein.cmp(&other.ein))
    }
}

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Entity").field("ein", &self.ein).finish()
    }
}
