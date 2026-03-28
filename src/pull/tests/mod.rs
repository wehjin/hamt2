use crate::db::{Datom, Db};
use crate::pull::pull::Pull;
use crate::pull::register::Register;
use crate::space::mem::MemSpace;
use common::Basis;
use crate::db::attr::Attr;
use crate::db::ent::Ent;
use crate::db::Val;

pub mod common;

#[test]
fn push_test() {
    let _register = Register::new().register::<Basis>().unwrap();
    let basis = Basis {
        symbol: "ABC".to_string(),
        shares: 100,
        price_each: 101,
        direction: -1,
    };
    let datoms = basis.into_datoms(27).expect("into_datoms");
    assert_eq!(
        vec![
            Datom::Add(Ent(27), Attr("basis", "symbol"), Val::from_str("ABC")),
            Datom::Add(Ent(27), Attr("basis", "shares"), Val::U32(100)),
            Datom::Add(Ent(27), Attr("basis", "price_each"), Val::U32(101)),
            Datom::Add(Ent(27), Attr("basis", "direction"), Val::U32(u32::MAX)),
        ],
        datoms
    );
}

#[test]
fn pull_trait() {
    let register = Register::new().register::<Basis>().unwrap();
    let attrs = register.to_attrs();
    // Push
    let (id, space) = {
        let basis = Basis {
            symbol: "ABC".to_string(),
            shares: 100,
            price_each: 100,
            direction: -1,
        };
        let mut db = Db::new(MemSpace::new(), attrs.clone()).unwrap();
        let id = db.max_eid().unwrap();
        let datoms = basis.into_datoms(id).unwrap();
        db = db.transact(datoms).unwrap();
        (id, db.close())
    };
    // Pull
    assert_eq!(
        Basis {
            symbol: "ABC".to_string(),
            shares: 100,
            price_each: 100,
            direction: -1,
        },
        Db::load(space, attrs).unwrap().pull::<Basis>(id).unwrap()
    );
}
