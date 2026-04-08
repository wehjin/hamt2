use crate::db::attr::Attr;
use crate::db::ent::Ent;
use crate::db::Ein;
use crate::db::Val;
use crate::db::{dat, Datom, Db};
use crate::pull::Pull;
use crate::space::mem::MemSpace;
use common::Basis;

pub mod common;

#[tokio::test]
async fn pull_test() {
    let space = {
        let basis = Basis {
            symbol: "ABC".to_string(),
            shares: 100,
            price_each: 101,
            direction: -1,
        };
        let ent = Ent::from(27);
        let mut db = Db::new(MemSpace::new(), Basis::attrs())
            .await
            .expect("Db::new");
        let datoms = basis.into_datoms(ent);
        db = db.transact(datoms).await.expect("db.transact");
        db.close()
    };
    {
        let db = Db::load(space, Basis::attrs()).await.expect("Db::load");
        assert_eq!(
            Basis {
                symbol: "ABC".to_string(),
                shares: 100,
                price_each: 101,
                direction: -1,
            },
            Basis::pull(&db, Ein(27)).await.expect("Basis::pull")
        )
    }
}

#[test]
fn push_test() {
    let basis = Basis {
        symbol: "ABC".to_string(),
        shares: 100,
        price_each: 101,
        direction: -1,
    };
    let ent = Ent::from(27);
    let datoms = basis.into_datoms(ent);
    assert_eq!(
        vec![
            Datom::Add(ent, Attr("basis/symbol"), dat(Val::from_str("ABC"))),
            Datom::Add(ent, Attr("basis/shares"), dat(Val::U32(100))),
            Datom::Add(ent, Attr("basis/price_each"), dat(Val::U32(101))),
            Datom::Add(ent, Attr("basis/direction"), dat(Val::U32(u32::MAX))),
        ],
        datoms
    );
}
