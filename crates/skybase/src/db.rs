use hamt2::ConnectError;
use hamt2::db::Db;
use hamt2::storage::MemDbStorage;
use sky_types::db::datum;
use sky_types::db::{Attr, Ent, Transact, val};

pub const ATTR_SKYBASE_VERSION: Attr = Attr("skybase/version");
pub async fn start_db() -> Result<Db<MemDbStorage>, ConnectError> {
    let db = Db::new(MemDbStorage::new(), [ATTR_SKYBASE_VERSION]).await?;
    let db = db
        .transact([datum::add(
            Ent::Temp("skybase"),
            ATTR_SKYBASE_VERSION,
            val("0.1"),
        )])
        .await?;
    Ok(db)
}
