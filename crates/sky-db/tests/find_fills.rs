use sky_db::DbQuery;
use sky_db::Transact;
use sky_db::create;
use sky_db::find::EntityFills;
use sky_db::{Attr, Fill, Val, dat, datom, ein};

fn attr_count() -> Attr {
    Attr::from("counter/count")
}

#[tokio::test]
async fn find_fills_works() {
    let ein = ein(300);

    let dir = tempfile::tempdir().unwrap();
    let mut peer = create(dir.path(), [attr_count()]).await.unwrap();
    peer.transact([datom::add(ein, attr_count(), dat(300))])
        .await
        .unwrap();

    let pod = peer.snapshot().await;
    let fills = pod.find(EntityFills(ein)).await;
    assert_eq!(fills, vec![Fill(attr_count(), Val::U32(300))]);
}
