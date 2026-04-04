use crate::db::attr::Attr;
use crate::db::{Db, Ent};
use crate::pull::pull::Pull;
use crate::space::Space;
use crate::QueryError;
use serde::{Deserialize, Serialize};
use crate::db::Eid;

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

    async fn pull<T: Space>(db: &Db<T>, eid: Eid) -> Result<Self, QueryError> {
        let symbol = db
            .find_val(Ent::Id(eid), Self::SYMBOL)
            .await?
            .expect("symbol");
        let shares = db
            .find_val(Ent::Id(eid), Self::SHARES)
            .await?
            .expect("shares");
        let price_each = db
            .find_val(Ent::Id(eid), Self::PRICE_EACH)
            .await?
            .expect("price_each");
        let direction = db
            .find_val(Ent::Id(eid), Self::DIRECTION)
            .await?
            .expect("direction");
        Ok(Self {
            symbol: symbol.as_str().to_string(),
            shares: shares.u32(),
            price_each: price_each.u32(),
            direction: direction.u32() as i32,
        })
    }
}
