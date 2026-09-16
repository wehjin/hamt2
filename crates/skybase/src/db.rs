use sky_db::ConnectError;
use sky_db::db::Db;
use sky_trie::storage::mem::MemStorage;
use sky_types::db::datom;
use sky_types::db::{Attr, Ent, Transact, val};

pub fn skybase_version() -> Attr {
    Attr::from("skybase/version")
}

pub async fn start_db() -> Result<Db<MemStorage>, ConnectError> {
    let db = Db::new(MemStorage::new(), ["skybase/version"]).await?;
    let db = db
        .transact([datom::add(
            Ent::Temp("skybase".to_string()),
            skybase_version(),
            val("0.1"),
        )])
        .await?;
    Ok(db)
}
