use hamt2::TransactError;
use hamt2::db::{Attr, Db, Ent, datom, val};
use hamt2::trie::base_storage::mem::MemBaseStorage;

pub const ATTR_SKYBASE_VERSION: Attr = Attr("skybase/version");
pub async fn start_db() -> Result<Db<MemBaseStorage>, TransactError> {
    let db = Db::new(MemBaseStorage::new(), [ATTR_SKYBASE_VERSION]).await?;
    db.transact([datom::add(
        Ent::Temp("skybase"),
        ATTR_SKYBASE_VERSION,
        val("0.1"),
    )])
    .await
}
