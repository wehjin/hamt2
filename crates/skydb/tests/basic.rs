use skydb::SkyDb;

#[test]
fn version_works() {
    let db = SkyDb::connect().unwrap();
    let version = db.version().unwrap();
    assert_eq!("0.1", version.as_str());
}
