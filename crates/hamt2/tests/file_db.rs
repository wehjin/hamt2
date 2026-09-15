use hamt2::db::Db;
use hamt2::query::DbQuery;
use hamt2::storage::FileDbStorage;
use sky_types::db::Transact;
use sky_types::db::Attr;
use sky_types::db::datom;
use sky_types::db::val;

pub const ATTR_COUNT: Attr = Attr("counter/count");
pub const ATTR_GREETING: Attr = Attr("speech/greeting");

#[tokio::test]
async fn file_db_works() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    {
        let storage = FileDbStorage::new(dir.path())?;
        let db = Db::new(storage, [ATTR_COUNT]).await?;
        let db = db.transact([datom::add(1, ATTR_COUNT, 1)]).await?;
        assert_eq!(Some(val(1)), db.find_val(1, ATTR_COUNT).await?);
        db.close();
    }
    {
        let storage = FileDbStorage::load(dir.path())?;
        let db = Db::load(storage, [ATTR_COUNT]).await?;
        assert_eq!(Some(val(1)), db.find_val(1, ATTR_COUNT).await?);
    }
    Ok(())
}

#[tokio::test]
async fn file_db_strings_work() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    {
        let storage = FileDbStorage::new(dir.path())?;
        let db = Db::new(storage, [ATTR_GREETING]).await?;
        let db = db.transact([datom::add(1, ATTR_GREETING, "hello")]).await?;
        assert_eq!(Some(val("hello")), db.find_val(1, ATTR_GREETING).await?);
        db.close();
    }
    {
        let storage = FileDbStorage::load(dir.path())?;
        let db = Db::load(storage, [ATTR_GREETING]).await?;
        assert_eq!(Some(val("hello")), db.find_val(1, ATTR_GREETING).await?);
    }
    Ok(())
}
