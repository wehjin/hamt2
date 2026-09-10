use skydb::{SkyViewer, start_db};

#[test]
fn version_works() {
    let viewer = pollster::block_on(async {
        let db = start_db().await.unwrap();
        SkyViewer::start(db.to_space())
    });
    let version = viewer.get_version();
    assert_eq!("0.1", version.as_str());
}
