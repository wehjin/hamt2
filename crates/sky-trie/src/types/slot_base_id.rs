use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SlotBaseId(pub i32);

impl Display for SlotBaseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}