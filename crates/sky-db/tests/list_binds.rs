use sky_db;
use sky_db::{Pod, Transact, datom, val};

#[tokio::test]
async fn list_binds_works() {
    let mut pod = Pod::new(["counter/count"]).await.unwrap();
    {
        let attribute = pod.get_attribute("counter/count").unwrap();
        let binds = attribute.list_binds();
        assert_eq!(binds, vec![]);
    }

    let datoms = [
        datom::add(33, "counter/count", 33),
        datom::add(37, "counter/count", 37),
    ];
    pod.transact(datoms).await.unwrap();
    {
        let attribute = pod.get_attribute("counter/count").unwrap();
        let binds = attribute.list_binds();
        let vals = binds
            .iter()
            .map(|bind| bind.val().clone())
            .collect::<Vec<_>>();
        assert_eq!(vals, vec![val(33), val(37)]);
    }
}
