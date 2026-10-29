use sky_db::{Attr, Transact, Val, create, dat, datom, ent};

#[tokio::test]
async fn ev_iter_test() -> anyhow::Result<()> {
    let count = || Attr::from("counter/count");
    let schema = vec![count()];
    let dir = tempfile::tempdir()?;
    let mut peer = create(dir.path(), schema.clone()).await?;
    peer.transact(vec![
        datom::add(ent(10), count(), dat(10)),
        datom::add(ent(11), count(), dat(Val::from(11))),
    ])
    .await?;

    let pod = peer.snapshot().await;
    let mut ev_vec = pod.ev_iter(count()).collect::<Vec<_>>();
    ev_vec.sort_by_key(|ev| ev.0);
    assert_eq!(vec![(10, Val::from(10)), (11, Val::from(11))], ev_vec);
    Ok(())
}
