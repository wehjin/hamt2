use hamt2::db::find::{AnyAttrIgnore, Find};
use hamt2::db::{datom, ein, val, Attr};
use hamt2::db::{Db, Txid};
use hamt2::space::mem::MemSpace;
use hamt2::LoadError;

pub const ATTR_COUNT: Attr = Attr("counter/count");
pub const ATTR_GREETING: Attr = Attr("speech/greeting");

#[tokio::test]
async fn load_works() -> anyhow::Result<()> {
    let space = MemSpace::new();
    let db = Db::new(space, [ATTR_COUNT]).await?;
    let db = db.transact([datom::add(1, ATTR_COUNT, 1)]).await?;
    let space = db.close();
    let db = Db::load(space, [ATTR_COUNT]).await?;
    assert_eq!(Some(val(1)), db.find_val(1, ATTR_COUNT).await?);
    Ok(())
}

#[tokio::test]
async fn load_fails_with_unknown_attribute() -> anyhow::Result<()> {
    let db = Db::new(MemSpace::new(), [ATTR_COUNT]).await?;
    let space = db.close();
    let result = Db::load(space, [ATTR_COUNT, ATTR_GREETING]).await;
    let Err(LoadError::UnknownAttr(ATTR_GREETING)) = result else {
        panic!("load should fail with unknown attr");
    };
    Ok(())
}

#[tokio::test]
async fn transact_and_pull_simple() -> anyhow::Result<()> {
    let db = Db::new(MemSpace::new(), [ATTR_COUNT]).await?;
    let db = db.transact([datom::add(15, ATTR_COUNT, 15)]).await?;
    assert_eq!(Some(val(15)), db.find_val(15, ATTR_COUNT).await?);
    Ok(())
}

#[tokio::test]
async fn entities_with_attr_works_for_single_entity() -> anyhow::Result<()> {
    let db = Db::new(MemSpace::new(), [ATTR_COUNT]).await?;
    let db = db.transact([datom::add(15, ATTR_COUNT, 15)]).await?;
    let eins = AnyAttrIgnore::new(ATTR_COUNT).apply_db(&db).await?;
    assert_eq!(vec![ein(15)], eins);
    Ok(())
}

#[tokio::test]
async fn entities_with_attr_works_for_two_entities() -> anyhow::Result<()> {
    let db = Db::new(MemSpace::new(), [ATTR_COUNT]).await?;
    let db = db
        .transact([datom::add(3, ATTR_COUNT, 4), datom::add(5, ATTR_COUNT, 6)])
        .await?;

    let mut eins = AnyAttrIgnore::new(ATTR_COUNT).apply_db(&db).await?;
    eins.sort();
    assert_eq!(vec![ein(3), ein(5)], eins);
    Ok(())
}

#[tokio::test]
async fn transact_assigns_id_to_temporary_ent() -> anyhow::Result<()> {
    let db = Db::new(MemSpace::new(), [ATTR_COUNT]).await?;
    let db = db
        .transact([datom::add("new_count", ATTR_COUNT, 35)])
        .await?;
    let db = db
        .transact([datom::add("new_count", ATTR_COUNT, 35)])
        .await?;
    let eins = AnyAttrIgnore::new(ATTR_COUNT).apply_db(&db).await?;
    assert_eq!(2, eins.len());
    Ok(())
}

#[tokio::test]
async fn test_multiple_entities() -> anyhow::Result<()> {
    // Construct a new database.
    let db = Db::new(MemSpace::new(), [ATTR_COUNT]).await?;
    assert_eq!(Txid::FLOOR, db.max_tx().await?);

    // Add a few datoms to different entities.
    let db = db
        .transact([
            datom::add(15, ATTR_COUNT, 15),
            datom::add(5, ATTR_COUNT, val(5)),
        ])
        .await?;
    assert_eq!(Txid::FLOOR + 1, db.max_tx().await?);
    assert_eq!(Some(val(15)), db.find_val(15, ATTR_COUNT).await?);
    assert_eq!(Some(val(5)), db.find_val(5, ATTR_COUNT).await?);

    // Discover the entities with an attribute.
    let mut eins = AnyAttrIgnore::new(ATTR_COUNT).apply_db(&db).await?;
    eins.sort();
    assert_eq!(vec![ein(5), ein(15)], eins);
    Ok(())
}
