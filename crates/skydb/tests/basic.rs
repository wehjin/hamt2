use skydb::SkyDb;

#[test]
fn version_works() {
    let db = SkyDb::connect().unwrap();
    let viewer = db.to_viewer();
    let version = viewer.version();
    assert_eq!("0.1", version.as_str());
}
