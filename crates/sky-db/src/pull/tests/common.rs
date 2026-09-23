use crate::db::Db;
use crate::pull::Pull;
use crate::traits::DbQuery;
use serde::{Deserialize, Serialize};
use sky_types::db::Datom;
use sky_types::db::QueryError;
use sky_types::db::datom;
use sky_types::db::{Attr, Ein, Ent};
use sky_types::storage::TrieEdit;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename = "basis")]
pub struct Basis {
    pub symbol: String,
    pub shares: u32,
    pub price_each: u32,
    pub direction: i32,
}

impl Basis {
    fn symbol() -> Attr {
        Attr::from("basis/symbol")
    }
    fn shares() -> Attr {
        Attr::from("basis/shares")
    }
    fn price_each() -> Attr {
        Attr::from("basis/price_each")
    }
    fn direction() -> Attr {
        Attr::from("basis/direction")
    }
}

impl<'a> Pull<'a> for Basis {
    fn attrs() -> Vec<Attr> {
        vec![
            Self::symbol(),
            Self::shares(),
            Self::price_each(),
            Self::direction(),
        ]
    }

    fn into_datoms(self, ent: Ent) -> Vec<Datom> {
        vec![
            datom::add(ent.clone(), Self::symbol(), self.symbol),
            datom::add(ent.clone(), Self::shares(), self.shares),
            datom::add(ent.clone(), Self::price_each(), self.price_each),
            datom::add(ent, Self::direction(), self.direction),
        ]
    }

    async fn pull<S: TrieEdit>(db: &Db<S>, eid: Ein) -> Result<Self, QueryError> {
        let symbol = db.find_val(eid, Self::symbol()).await?.expect("symbol");
        let shares = db.find_val(eid, Self::shares()).await?.expect("shares");
        let price_each = db
            .find_val(eid, Self::price_each())
            .await?
            .expect("price_each");
        let direction = db
            .find_val(eid, Self::direction())
            .await?
            .expect("direction");
        Ok(Self {
            symbol: symbol.as_str().to_string(),
            shares: shares.u32(),
            price_each: price_each.u32(),
            direction: direction.i32(),
        })
    }
}
