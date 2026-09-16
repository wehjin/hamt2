use sky_db::ConnectError;
use sky_db::db::Db;
use sky_trie::storage::mem::MemStorage;
use sky_types::db::datom;
use sky_types::db::{Attr, Ent, Transact, val};

pub const ATTR_SKYBASE_VERSION: Attr = Attr("skybase/version");
pub async fn start_db() -> Result<Db<MemStorage>, ConnectError> {
    let db = Db::new(MemStorage::new(), [ATTR_SKYBASE_VERSION]).await?;
    let db = db
        .transact([datom::add(
            Ent::Temp("skybase"),
            ATTR_SKYBASE_VERSION,
            val("0.1"),
        )])
        .await?;
    Ok(db)
}
