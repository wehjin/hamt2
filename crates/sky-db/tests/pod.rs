use sky_db::DbQuery;
use sky_db::Pod;
use sky_db::Transact;
use sky_db::datom;
use sky_db::find::EinsWithAttr;
use sky_db::{Attr, val};

fn attr_count() -> Attr {
    Attr::from("counter/count")
}

#[tokio::test]
async fn pod_works() -> anyhow::Result<()> {
    let mut pod = Pod::new([attr_count()]).await?;
    pod.transact([
        datom::add(1, attr_count(), val(10)),
        datom::add(2, attr_count(), val(20)),
        datom::add(3, attr_count(), val(30)),
    ])
    .await?;

    let mut eins = pod.find(EinsWithAttr::new(attr_count())).await;
    eins.sort();

    for ein in eins {
        let count = pod.find_val(ein, attr_count()).await?;
        println!("entity {:?} -> {:?}", ein, count);
    }

    Ok(())
}
