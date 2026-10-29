use crate::datom;
use crate::pull::Pull;
use crate::{Attr, Ein, Ent, Val, dat};
use crate::{Transact, create};
use common::Basis;

pub mod common;

#[tokio::test]
async fn pull_test() {
    let basis = Basis {
        symbol: "ABC".to_string(),
        shares: 100,
        price_each: 101,
        direction: -1,
    };
    let ent = Ent::from(27);
    let dir = tempfile::tempdir().expect("tempdir");
    let mut peer = create(dir.path(), Basis::attrs())
        .await
        .expect("create");
    peer.transact(basis.into_datoms(ent))
        .await
        .expect("pod.transact");

    let pod = peer.snapshot().await;
    assert_eq!(
        Basis {
            symbol: "ABC".to_string(),
            shares: 100,
            price_each: 101,
            direction: -1,
        },
        Basis::pull(&pod, Ein(27)).await.expect("Basis::pull")
    )
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
