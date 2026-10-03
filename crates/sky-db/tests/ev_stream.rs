use futures::StreamExt;
use sky_db::{Attr, Pod, Transact, Val, dat, datom, ent};
use sky_trie::SkyTrie;

#[tokio::test]
async fn ev_stream_test() -> anyhow::Result<()> {
    let count = || Attr::from("counter/count");
    let schema = vec![count()];
    let storage = SkyTrie::new();
    let mut db = Pod::new(storage, schema.clone()).await?;
    db.transact(vec![
        datom::add(ent(10), count(), dat(10)),
        datom::add(ent(11), count(), dat(Val::from(11))),
    ])
    .await?;

    let ev_stream = db.ev_stream(count());
    let mut ev_vec = ev_stream.collect::<Vec<_>>().await;
    ev_vec.sort_by_key(|ev| ev.0);
    assert_eq!(vec![(10, Val::from(10)), (11, Val::from(11))], ev_vec);
    Ok(())
}
