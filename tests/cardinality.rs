use hamt2::db::attr_spec::AttrSpec;
use hamt2::db::cardinality::Cardinality;
use hamt2::db::find::{EinAttrAny, Find};
use hamt2::db::{dat, datom, ein, val, Attr, Db};
use hamt2::space::mem::MemSpace;

#[tokio::test]
async fn test_cardinality_one() -> anyhow::Result<()> {
    let space = MemSpace::new();
    const COUNT: Attr = Attr("counter/count");
    let schema = vec![AttrSpec {
        attr: COUNT,
        cardinality: Cardinality::One,
    }];
    let mut db = Db::new(space, schema).await?;
    let ein = ein(100);
    db = db.transact([datom(ein, COUNT, dat(100))]).await?;
    db = db.transact([datom(ein, COUNT, dat(101))]).await?;
    db = db.transact([datom(ein, COUNT, dat(102))]).await?;
    let vals = EinAttrAny::new(ein, COUNT).apply_db(&db).await?;
    assert_eq!(vec![val(102)], vals);
    Ok(())
}

#[tokio::test]
async fn test_cardinality_many() -> anyhow::Result<()> {
    let space = MemSpace::new();
    const COUNT: Attr = Attr("counter/count");
    let schema = vec![AttrSpec {
        attr: COUNT,
        cardinality: Cardinality::Many,
    }];
    let mut db = Db::new(space, schema).await?;
    let ein = ein(100);
    db = db.transact([datom(ein, COUNT, dat(100))]).await?;
    db = db.transact([datom(ein, COUNT, dat(101))]).await?;
    db = db.transact([datom(ein, COUNT, dat(102))]).await?;
    let mut vals = EinAttrAny::new(ein, COUNT).apply_db(&db).await?;
    vals.sort();
    assert_eq!(vec![val(100), val(101), val(102)], vals);
    Ok(())
}
