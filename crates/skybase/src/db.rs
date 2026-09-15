use hamt2::ConnectError;
use hamt2::db::db::types::datom;
use hamt2::db::{Attr, Db, Ent, val};
use hamt2::storage::MemDbStorage;

pub const ATTR_SKYBASE_VERSION: Attr = Attr("skybase/version");
pub async fn start_db() -> Result<Db<MemDbStorage>, ConnectError> {
    let db = Db::new(MemDbStorage::new(), [ATTR_SKYBASE_VERSION]).await?;
    let db = db
        .transact([datom::add(
            Ent::Temp("skybase"),
            ATTR_SKYBASE_VERSION,
            val("0.1"),
        )])
        .await?;
    Ok(db)
}
