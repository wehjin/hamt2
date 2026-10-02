use sky_db::db::Db;
use sky_db::db::attr_spec::AttrSpec;
use sky_db::db::cardinality::Cardinality;
use sky_db::find::ValsInSlot;
use sky_db::traits::DbQuery;
use sky_types::db::Transact;
use sky_types::db::datom;
use sky_types::db::{Attr, val};
use sky_types::storage::MemLoad;
use sky_types::storage::load::StoreLoad;

#[tokio::test]
async fn one() -> anyhow::Result<()> {
    let count = || Attr::from("counter/count");
    let schema = [AttrSpec {
        attr: count(),
        cardinality: Cardinality::One,
    }];
    let mut db = Db::new(MemLoad::new(), schema).await?;
    db.transact([datom::add(100, count(), 100)]).await?;
    db.transact([datom::add(100, count(), 101)]).await?;
    db.transact([datom::add(100, count(), 102)]).await?;
    let vals = db.find(ValsInSlot::new(100, count())).await;
    assert_eq!(vec![val(102)], vals);

    db.transact([datom::del(100, count(), 102)]).await?;
    let vals = db.find(ValsInSlot::new(100, count())).await;
    assert!(vals.is_empty());
    Ok(())
}

#[tokio::test]
async fn many() -> anyhow::Result<()> {
    let count = || Attr::from("counter/count");
    let schema = [AttrSpec {
        attr: count(),
        cardinality: Cardinality::Many,
    }];
    let mut db = Db::new(MemLoad::new(), schema).await?;
    db.transact([datom::add(100, count(), 100)]).await?;
    db.transact([datom::add(100, count(), 101)]).await?;
    db.transact([datom::add(100, count(), 102)]).await?;
    let mut vals = db.find(ValsInSlot::new(100, count())).await;
    vals.sort();
    assert_eq!(vec![val(100), val(101), val(102)], vals);

    db.transact([datom::del(100, count(), 101)]).await?;
    let mut vals = db.find(ValsInSlot::new(100, count())).await;
    vals.sort();
    assert_eq!(vec![val(100), val(102)], vals);
    Ok(())
}
