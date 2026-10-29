use sky_db::{Transact, create, datom, val};

#[tokio::test]
async fn list_binds_works() {
    let dir = tempfile::tempdir().unwrap();
    let mut pod = create(dir.path(), ["counter/count"]).await.unwrap();
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
