use sky_db::{Attr, Pod, Transact, Val, dat, datom, ent};

#[tokio::test]
async fn ev_iter_test() -> anyhow::Result<()> {
    let count = || Attr::from("counter/count");
    let schema = vec![count()];
    let mut db = Pod::new(schema.clone()).await?;
    db.transact(vec![
        datom::add(ent(10), count(), dat(10)),
        datom::add(ent(11), count(), dat(Val::from(11))),
    ])
    .await?;

    let mut ev_vec = db.ev_iter(count()).collect::<Vec<_>>();
    ev_vec.sort_by_key(|ev| ev.0);
    assert_eq!(vec![(10, Val::from(10)), (11, Val::from(11))], ev_vec);
    Ok(())
}
