use crate::db::{Attr, Val};
use crate::pull::errors::BuildError;
use crate::pull::pull::Pull;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

impl Pull for Basis {
    type Target = Self;
    fn attrs() -> Vec<Attr> {
        vec![
            Self::SYMBOL,
            Self::SHARES,
            Self::PRICE_EACH,
            Self::DIRECTION,
        ]
    }
    fn build(bindings: Vec<(Attr, Option<Val>)>) -> Result<Self::Target, BuildError> {
        let mut map = bindings
            .into_iter()
            .filter_map(|(attr, val)| val.map(|val| (attr, val)))
            .collect::<HashMap<_, _>>();
        let symbol = map
            .remove(&Self::SYMBOL)
            .and_then(|v| v.try_into_string())
            .ok_or_else(|| anyhow::Error::msg("missing symbol"))?;
        let shares = map
            .remove(&Self::SHARES)
            .and_then(|v| v.try_into_u32())
            .ok_or_else(|| anyhow::Error::msg("missing shares"))?;
        let price_each = map
            .remove(&Self::PRICE_EACH)
            .and_then(|v| v.try_into_u32())
            .ok_or_else(|| anyhow::Error::msg("missing price_each"))?;
        let direction = map
            .remove(&Self::DIRECTION)
            .and_then(|v| v.try_into_u32())
            .ok_or_else(|| anyhow::Error::msg("missing direction"))?;
        let basis = Basis {
            symbol,
            shares,
            price_each,
            direction: direction as i32,
        };
        Ok(basis)
    }
}
