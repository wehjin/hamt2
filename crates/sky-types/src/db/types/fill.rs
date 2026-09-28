use crate::db::{Attr, Val};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Fill(pub Attr, pub Val);
