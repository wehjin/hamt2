use sky_db::DbQuery;
use sky_db::Pod;
use sky_db::find::EntityFills;
use sky_db::{Attr, Fill, Transact, Val, dat, datom, ein};

fn attr_count() -> Attr {
    Attr::from("counter/count")
}

#[tokio::test]
async fn find_fills_works() {
    let ein = ein(300);

    let mut db = Pod::new([attr_count()]).await.unwrap();
    db.transact([datom::add(ein, attr_count(), dat(300))])
        .await
        .unwrap();

    let fills = db.find(EntityFills(ein)).await;
    assert_eq!(fills, vec![Fill(attr_count(), Val::U32(300))]);
}
