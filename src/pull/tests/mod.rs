use crate::db::attr::Attr;
use crate::db::ent::Ent;
use crate::db::{Datom, Db};
use crate::db::{Eid, Val};
use crate::pull::pull::Pull;
use crate::pull::register::Register;
use crate::space::mem::MemSpace;
use common::Basis;

pub mod common;

#[test]
fn pull_test() {
    let space = {
        let basis = Basis {
            symbol: "ABC".to_string(),
            shares: 100,
            price_each: 101,
            direction: -1,
        };
        let ent = Ent::from(27);
        let mut db = Db::new(MemSpace::new(), Basis::attrs()).expect("Db::new");
        let datoms = basis.into_datoms(ent).expect("into_datoms");
        db = db.transact(datoms).expect("db.transact");
        db.close()
    };
    {
        let db = Db::load(space, Basis::attrs()).expect("Db::load");
        assert_eq!(
            Basis {
                symbol: "ABC".to_string(),
                shares: 100,
                price_each: 101,
                direction: -1,
            },
            db.pull::<Basis>(Eid(27)).expect("db.pull2")
        )
    }
}

#[test]
fn push_test() {
    let _register = Register::new().register::<Basis>().unwrap();
    let basis = Basis {
        symbol: "ABC".to_string(),
        shares: 100,
        price_each: 101,
        direction: -1,
    };
    let ent = Ent::from(27);
    let datoms = basis.into_datoms(ent).expect("into_datoms");
    assert_eq!(
        vec![
            Datom::Add(ent, Attr("basis", "symbol"), Val::from_str("ABC")),
            Datom::Add(ent, Attr("basis", "shares"), Val::U32(100)),
            Datom::Add(ent, Attr("basis", "price_each"), Val::U32(101)),
            Datom::Add(ent, Attr("basis", "direction"), Val::U32(u32::MAX)),
        ],
        datoms
    );
}
