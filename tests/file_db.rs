use hamt2::db::Attr;
use hamt2::db::Db;
use hamt2::db::{datom, val};
use hamt2::space::file::FileSpace;

pub const ATTR_COUNT: Attr = Attr("counter/count");
pub const ATTR_GREETING: Attr = Attr("speech/greeting");

#[tokio::test]
async fn file_db_works() -> anyhow::Result<()> {
    let file = tempfile::NamedTempFile::new()?;
    {
        let space = FileSpace::new(&file).await?;
        let db = Db::new(space, [ATTR_COUNT]).await?;
        let db = db.transact([datom::add(1, ATTR_COUNT, 1)]).await?;
        assert_eq!(Some(val(1)), db.find_val(1, ATTR_COUNT).await?);
    }
    {
        let space = FileSpace::load(&file).await?;
        let db = Db::load(space, [ATTR_COUNT]).await?;
        assert_eq!(Some(val(1)), db.find_val(1, ATTR_COUNT).await?);
    }
    Ok(())
}

#[tokio::test]
async fn file_db_strings_work() -> anyhow::Result<()> {
    let file = tempfile::NamedTempFile::new()?;
    {
        let space = FileSpace::new(&file).await?;
        let db = Db::new(space, [ATTR_GREETING]).await?;
        let db = db.transact([datom::add(1, ATTR_GREETING, "hello")]).await?;
        assert_eq!(Some(val("hello")), db.find_val(1, ATTR_GREETING).await?);
    }
    {
        let space = FileSpace::load(&file).await?;
        let db = Db::load(space, [ATTR_GREETING]).await?;
        assert_eq!(Some(val("hello")), db.find_val(1, ATTR_GREETING).await?);
    }
    Ok(())
}
