use sky_db::db::Db;
use sky_db::db::Txid;
use sky_db::find::EinsWithAttr;
use sky_db::traits::DbQuery;
use sky_types::db::Transact;
use sky_types::db::datom;
use sky_types::db::{Attr, ein, val};
use sky_types::storage::mem_edit_new;

pub fn attr_count() -> Attr {
    Attr::from("counter/count")
}

#[tokio::test]
async fn load_works() -> anyhow::Result<()> {
    let storage = mem_edit_new();
    let db = Db::new(storage, [attr_count()]).await?;
    let db = db.transact([datom::add(1, attr_count(), 1)]).await?;
    let storage = db.close();
    let db = Db::load(storage).await;
    assert_eq!(Some(val(1)), db.find_val(1, attr_count()).await?);
    Ok(())
}

#[tokio::test]
async fn transact_and_pull_simple() -> anyhow::Result<()> {
    let db = Db::new(mem_edit_new(), [attr_count()]).await?;
    let db = db.transact([datom::add(15, attr_count(), 15)]).await?;
    assert_eq!(Some(val(15)), db.find_val(15, attr_count()).await?);
    Ok(())
}

#[tokio::test]
async fn entities_with_attr_works_for_single_entity() -> anyhow::Result<()> {
    let db = Db::new(mem_edit_new(), [attr_count()]).await?;
    let db = db.transact([datom::add(15, attr_count(), 15)]).await?;
    let eins = db.find(EinsWithAttr::new(attr_count())).await;
    assert_eq!(vec![ein(15)], eins);
    Ok(())
}

#[tokio::test]
async fn entities_with_attr_works_for_two_entities() -> anyhow::Result<()> {
    let db = Db::new(mem_edit_new(), [attr_count()]).await?;
    let db = db
        .transact([
            datom::add(3, attr_count(), 4),
            datom::add(5, attr_count(), 6),
        ])
        .await?;

    let mut eins = db.find(EinsWithAttr::new(attr_count())).await;
    eins.sort();
    assert_eq!(vec![ein(3), ein(5)], eins);
    Ok(())
}

#[tokio::test]
async fn transact_assigns_id_to_temporary_ent() -> anyhow::Result<()> {
    let db = Db::new(mem_edit_new(), [attr_count()]).await?;
    let db = db
        .transact([datom::add("new_count", attr_count(), 35)])
        .await?;
    let db = db
        .transact([datom::add("new_count", attr_count(), 35)])
        .await?;
    let eins = db.find(EinsWithAttr::new(attr_count())).await;
    assert_eq!(2, eins.len());
    Ok(())
}

#[tokio::test]
async fn test_multiple_entities() -> anyhow::Result<()> {
    // Construct a new database.
    let db = Db::new(mem_edit_new(), [attr_count()]).await?;
    assert_eq!(Txid::FLOOR, db.max_tx().await?);

    // Add a few datoms to different entities.
    let db = db
        .transact([
            datom::add(15, attr_count(), 15),
            datom::add(5, attr_count(), val(5)),
        ])
        .await?;
    assert_eq!(Txid::FLOOR + 1, db.max_tx().await?);
    assert_eq!(Some(val(15)), db.find_val(15, attr_count()).await?);
    assert_eq!(Some(val(5)), db.find_val(5, attr_count()).await?);

    // Discover the entities with an attribute.
    let mut eins = db.find(EinsWithAttr::new(attr_count())).await;
    eins.sort();
    assert_eq!(vec![ein(5), ein(15)], eins);
    Ok(())
}
