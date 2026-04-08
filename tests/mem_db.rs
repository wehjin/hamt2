use hamt2::db::find::{AnyAttrIgnore, Find};
use hamt2::db::{dat, datom, ein, ent, Attr, Val};
use hamt2::db::{Db, Txid};
use hamt2::space::mem::MemSpace;
use hamt2::LoadError;

pub const ATTR_COUNT: Attr = Attr("counter/count");
pub const ATTR_GREETING: Attr = Attr("speech/greeting");

#[tokio::test]
async fn load_works() {
    let space = Db::new(MemSpace::new(), vec![ATTR_COUNT])
        .await
        .expect("new db")
        .transact(vec![datom(ent(1), ATTR_COUNT, dat(Val::U32(1)))])
        .await
        .expect("transact")
        .close();
    let db = Db::load(space, vec![ATTR_COUNT]).await.expect("load db");
    assert_eq!(
        Some(Val::U32(1)),
        db.find_val(ein(1), ATTR_COUNT).await.expect("find_val")
    );
}

#[tokio::test]
async fn load_fails_with_unknown_attribute() {
    let space = Db::new(MemSpace::new(), vec![ATTR_COUNT])
        .await
        .expect("new db")
        .close();
    let result = Db::load(space, vec![ATTR_COUNT, ATTR_GREETING]).await;
    let Err(LoadError::UnknownAttr(ATTR_GREETING)) = result else {
        panic!("load should fail with unknown attr");
    };
}

#[tokio::test]
async fn transact_and_pull_simple() {
    let db = Db::new(MemSpace::new(), vec![ATTR_COUNT])
        .await
        .expect("new db");
    let db = db
        .transact(vec![datom(ent(15), ATTR_COUNT, dat(Val::U32(15)))])
        .await
        .expect("transact");
    let v15 = db.find_val(ein(15), ATTR_COUNT).await.expect("find_val");
    assert_eq!(Some(Val::U32(15)), v15);
}

#[tokio::test]
async fn entities_with_attr_works_for_single_entity() {
    let db = Db::new(MemSpace::new(), vec![ATTR_COUNT])
        .await
        .expect("new db");
    let db = db
        .transact(vec![datom(ent(15), ATTR_COUNT, dat(Val::U32(15)))])
        .await
        .expect("transact");

    let eins = AnyAttrIgnore::new(ATTR_COUNT)
        .apply_db(&db)
        .await
        .expect("apply");
    assert_eq!(vec![ein(15)], eins);
}

#[tokio::test]
async fn entities_with_attr_works_for_two_entities() {
    let db = Db::new(MemSpace::new(), vec![ATTR_COUNT])
        .await
        .expect("new db");
    let db = db
        .transact(vec![
            datom(ent(3), ATTR_COUNT, dat(Val::U32(4))),
            datom(ent(5), ATTR_COUNT, dat(Val::U32(6))),
        ])
        .await
        .expect("transact");

    let mut eins = AnyAttrIgnore::new(ATTR_COUNT)
        .apply_db(&db)
        .await
        .expect("apply");
    eins.sort();
    assert_eq!(vec![ein(3), ein(5)], eins);
}

#[tokio::test]
async fn transact_assigns_id_to_ent() {
    let mut db = Db::new(MemSpace::new(), vec![ATTR_COUNT])
        .await
        .expect("new db");
    db = db
        .transact(vec![datom(ent("new_count"), ATTR_COUNT, dat(Val::U32(35)))])
        .await
        .expect("transact");
    db = db
        .transact(vec![datom(ent("new_count"), ATTR_COUNT, dat(Val::U32(35)))])
        .await
        .expect("transact");

    let eins = AnyAttrIgnore::new(ATTR_COUNT)
        .apply_db(&db)
        .await
        .expect("find");
    assert_eq!(2, eins.len());
}

#[tokio::test]
async fn transact_and_pull_works() {
    // Construct a new database.
    let db = Db::new(MemSpace::new(), vec![ATTR_COUNT])
        .await
        .expect("new db");
    assert_eq!(Txid::FLOOR, db.max_tx().await.expect("max_tx"));

    // Add a few datoms.
    let db = db
        .transact(vec![
            datom(ent(15), ATTR_COUNT, dat(Val::U32(15))),
            datom(ent(5), ATTR_COUNT, dat(Val::U32(5))),
        ])
        .await
        .expect("transact");

    let max_tx = db.max_tx().await.expect("max_tx");
    assert_eq!(Txid::FLOOR + 1, max_tx);

    let v15 = db.find_val(ein(15), ATTR_COUNT).await.expect("find_val");
    assert_eq!(Some(Val::U32(15)), v15);

    let v5 = db.find_val(ein(5), ATTR_COUNT).await.expect("find_val");
    assert_eq!(Some(Val::U32(5)), v5);

    // Discover the entities with an attribute.
    let mut eins = AnyAttrIgnore::new(ATTR_COUNT)
        .apply_db(&db)
        .await
        .expect("apply");
    eins.sort();
    assert_eq!(vec![ein(5), ein(15)], eins);
}
