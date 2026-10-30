use sky_db::DbQuery;
use sky_db::Transact;
use sky_db::create;
use sky_db::datom;
use sky_db::find::ValsInSlot;
use sky_db::{Attr, Val, ein, ent};

fn advisor() -> Attr {
    Attr::from("member/advisor")
}

#[tokio::test]
async fn ref_round_trips_as_ein() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let mut peer = create(dir.path(), [advisor()]).await?;
    peer.transact([datom::add(ein(1), advisor(), ent(ein(42)))]).await?;
    let pod = peer.snapshot().await;
    let vals = pod.find(ValsInSlot::new(ein(1), advisor())).await;
    assert_eq!(vec![Val::Ein(ein(42))], vals);
    Ok(())
}
