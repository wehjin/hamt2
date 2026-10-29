use sky_db::DbQuery;
use sky_db::Transact;
use sky_db::create;
use sky_db::datom;
use sky_db::find::EinsWithAttr;
use sky_db::join;
use sky_db::{Attr, val};

fn attr_count() -> Attr {
    Attr::from("counter/count")
}

#[tokio::test]
async fn two_peers_converge() -> anyhow::Result<()> {
    let dir_a = tempfile::tempdir()?;
    let dir_b = tempfile::tempdir()?;

    // Peer A creates the repo and transacts.
    let mut peer_a = create(dir_a.path(), [attr_count()]).await?;
    peer_a.transact([datom::add(1, attr_count(), val(10))]).await?;

    // Peer B joins via a fresh ticket from A.
    let ticket = peer_a.ticket().await?;
    let peer_b = join(dir_b.path(), ticket).await?;

    // Both converge on the same index.
    let pod_a = peer_a.snapshot().await;
    let pod_b = peer_b.snapshot().await;
    let mut eins_a = pod_a.find(EinsWithAttr::new(attr_count())).await;
    eins_a.sort();
    let mut eins_b = pod_b.find(EinsWithAttr::new(attr_count())).await;
    eins_b.sort();
    assert_eq!(eins_a, eins_b);
    assert_eq!(Some(val(10)), pod_b.find_val(1, attr_count()).await?);

    Ok(())
}

#[tokio::test]
async fn live_updates() -> anyhow::Result<()> {
    let dir_a = tempfile::tempdir()?;
    let dir_b = tempfile::tempdir()?;

    let mut peer_a = create(dir_a.path(), [attr_count()]).await?;
    let ticket = peer_a.ticket().await?;
    let peer_b = join(dir_b.path(), ticket).await?;

    // A transacts after B has already joined.
    peer_a.transact([datom::add(1, attr_count(), val(10))]).await?;

    // B's snapshot should eventually reflect A's transact (via the background loop).
    for _ in 0..100 {
        let pod_b = peer_b.snapshot().await;
        if pod_b.find_val(1, attr_count()).await? == Some(val(10)) {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    panic!("timed out waiting for live update");
}
