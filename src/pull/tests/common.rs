use crate::db::attr::Attr;
use crate::pull::pull::Pull;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename = "basis")]
pub struct Basis {
    pub symbol: String,
    pub shares: u32,
    pub price_each: u32,
    pub direction: i32,
}

impl Basis {
    const SYMBOL: Attr = Attr("basis", "symbol");
    const SHARES: Attr = Attr("basis", "shares");
    const PRICE_EACH: Attr = Attr("basis", "price_each");
    const DIRECTION: Attr = Attr("basis", "direction");
}

impl<'a> Pull<'a> for Basis {
    fn attrs() -> Vec<Attr> {
        vec![
            Self::SYMBOL,
            Self::SHARES,
            Self::PRICE_EACH,
            Self::DIRECTION,
        ]
    }
}
