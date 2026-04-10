use crate::db::{datom, Attr, Datom, Db, Ein, Ent};
use crate::pull::Pull;
use crate::space::Space;
use crate::QueryError;
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
    const SYMBOL: Attr = Attr("basis/symbol");
    const SHARES: Attr = Attr("basis/shares");
    const PRICE_EACH: Attr = Attr("basis/price_each");
    const DIRECTION: Attr = Attr("basis/direction");
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

    fn into_datoms(self, ent: Ent) -> Vec<Datom> {
        vec![
            datom::add(ent, Self::SYMBOL, self.symbol),
            datom::add(ent, Self::SHARES, self.shares),
            datom::add(ent, Self::PRICE_EACH, self.price_each),
            datom::add(ent, Self::DIRECTION, self.direction),
        ]
    }

    async fn pull<T: Space>(db: &Db<T>, eid: Ein) -> Result<Self, QueryError> {
        let symbol = db.find_val(eid, Self::SYMBOL).await?.expect("symbol");
        let shares = db.find_val(eid, Self::SHARES).await?.expect("shares");
        let price_each = db
            .find_val(eid, Self::PRICE_EACH)
            .await?
            .expect("price_each");
        let direction = db.find_val(eid, Self::DIRECTION).await?.expect("direction");
        Ok(Self {
            symbol: symbol.as_str().to_string(),
            shares: shares.u32(),
            price_each: price_each.u32(),
            direction: direction.u32() as i32,
        })
    }
}
