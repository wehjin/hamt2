use crate::db::{Attr, Datom, Db, Ent, Val};
use crate::pull::errors::BuildError;
use crate::pull::register::Register;
use crate::pull::Pull;
use crate::space::mem::MemSpace;
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq)]
struct Basis {
    symbol: String,
    shares: u32,
    price_each: u32,
}

impl Basis {
    const SYMBOL: Attr = Attr("basis/symbol");
    const SHARES: Attr = Attr("basis/shares");
    const PRICE_EACH: Attr = Attr("basis/price_each");
}
impl Pull for Basis {
    type Target = Self;
    fn attrs() -> Vec<Attr> {
        vec![Self::SYMBOL, Self::SHARES, Self::PRICE_EACH]
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
        let basis = Basis {
            symbol,
            shares,
            price_each,
        };
        Ok(basis)
    }
    fn to_datom(&self, id: i32) -> Vec<Datom> {
        vec![
            Datom::Add(Ent(id), Self::SYMBOL, Val::String(self.symbol.clone())),
            Datom::Add(Ent(id), Self::SHARES, Val::U32(self.shares)),
            Datom::Add(Ent(id), Self::PRICE_EACH, Val::U32(self.price_each)),
        ]
    }
}

#[test]
fn pull_test() {
    let register = Register::new().register::<Basis>().unwrap();
    let basis = Basis {
        symbol: "ABC".to_string(),
        shares: 100,
        price_each: 100,
    };
    // Push
    let (space, id) = {
        let mut db = Db::new(MemSpace::new(), register.to_attrs()).unwrap();
        let id = db.max_eid().unwrap();
        db = db.transact(basis.to_datom(id)).unwrap();
        let space = db.close();
        (space, id)
    };
    // Pull
    let db = Db::load(space, register.to_attrs()).unwrap();
    assert_eq!(basis, db.pull::<Basis>(id).unwrap());
}
