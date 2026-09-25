use sky_types::db::Attr;

pub fn attr_count() -> Attr {
    Attr::from("counter/count")
}
pub fn attr_greeting() -> Attr {
    Attr::from("speech/greeting")
}

#[ignore]
#[tokio::test]
async fn file_db_works() -> anyhow::Result<()> {
    let _dir = tempfile::tempdir()?;
    {
        // let storage = FileTrieEdit::new(dir.path()).await?;
        // let db = Db::new(storage, [attr_count()]).await?;
        // let db = db.transact([datom::add(1, attr_count(), 1)]).await?;
        // assert_eq!(Some(val(1)), db.find_val(1, attr_count()).await?);
        // db.close();
    }
    {
        // let storage = FileTrieEdit::load(dir.path()).await?;
        // let db = Db::load(storage).await;
        // assert_eq!(Some(val(1)), db.find_val(1, attr_count()).await?);
    }
    Ok(())
}

#[ignore]
#[tokio::test]
async fn file_db_strings_work() -> anyhow::Result<()> {
    let _dir = tempfile::tempdir()?;
    {
        // let storage = FileTrieEdit::new(dir.path()).await?;
        // let db = Db::new(storage, [attr_greeting()]).await?;
        // let db = db
        //     .transact([datom::add(1, attr_greeting(), "hello")])
        //     .await?;
        // assert_eq!(Some(val("hello")), db.find_val(1, attr_greeting()).await?);
        // db.close();
    }
    {
        // let storage = FileTrieEdit::load(dir.path()).await?;
        // let db = Db::load(storage).await;
        // assert_eq!(Some(val("hello")), db.find_val(1, attr_greeting()).await?);
    }
    Ok(())
}
