use hamt2::db::{Attr, Db, Ent, val};
use hamt2::storage::MemDbStorage;
use hamt2::{TransactError, datom};

pub const ATTR_SKYBASE_VERSION: Attr = Attr("skybase/version");
pub async fn start_db() -> Result<Db<MemDbStorage>, TransactError> {
    let db = Db::new(MemDbStorage::new(), [ATTR_SKYBASE_VERSION]).await?;
    db.transact([datom::add(
        Ent::Temp("skybase"),
        ATTR_SKYBASE_VERSION,
        val("0.1"),
    )])
    .await
}
