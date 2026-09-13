use hamt2::TransactError;
use hamt2::db::{Attr, Db, Ent, datom, val};
use hamt2::trie::prelude::MemTrieStorage;

pub const ATTR_SKYBASE_VERSION: Attr = Attr("skybase/version");
pub async fn start_db() -> Result<Db<MemTrieStorage>, TransactError> {
    let db = Db::new(MemTrieStorage::new(), [ATTR_SKYBASE_VERSION]).await?;
    db.transact([datom::add(
        Ent::Temp("skybase"),
        ATTR_SKYBASE_VERSION,
        val("0.1"),
    )])
    .await
}
