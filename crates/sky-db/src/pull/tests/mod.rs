use crate::db::Db;
use crate::pull::Pull;
use common::Basis;
use sky_types::db::Transact;
use sky_types::db::datom;
use sky_types::db::{Attr, Ein, Ent, Val, dat};
use sky_types::storage::Mem;

pub mod common;

#[tokio::test]
async fn pull_test() {
    let storage = {
        let basis = Basis {
            symbol: "ABC".to_string(),
            shares: 100,
            price_each: 101,
            direction: -1,
        };
        let ent = Ent::from(27);
        let mut db = Db::new(Mem::new(), Basis::attrs()).await.expect("Db::new");
        db.transact(basis.into_datoms(ent))
            .await
            .expect("db.transact");
        db.close()
    };
    {
        let db = Db::load(storage).await;
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
    let datoms = basis.into_datoms(ent.clone());
    assert_eq!(
        vec![
            datom::add(
                ent.clone(),
                Attr::from("basis/symbol"),
                dat(Val::from_str("ABC"))
            ),
            datom::add(ent.clone(), Attr::from("basis/shares"), dat(Val::U32(100))),
            datom::add(
                ent.clone(),
                Attr::from("basis/price_each"),
                dat(Val::U32(101))
            ),
            datom::add(ent, Attr::from("basis/direction"), dat(Val::U32(u32::MAX))),
        ],
        datoms
    );
}
