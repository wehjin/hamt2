use hamt2::TransactError;
use hamt2::db::query::DbQuery;
use hamt2::db::reader::DbReader;
use hamt2::db::{Attr, Db, datom, val};
use hamt2::trie::base_storage::mem::{MemBaseStorage, MemReadStorage};

pub const ATTR_SKYBASE_VERSION: Attr = Attr("skybase/version");
pub const EID_SKYBASE: i32 = 1;

pub async fn start_db() -> Result<Db<MemBaseStorage>, TransactError> {
    let db = Db::new(MemBaseStorage::new(), [ATTR_SKYBASE_VERSION]).await?;
    db.transact([datom::add(EID_SKYBASE, ATTR_SKYBASE_VERSION, val("0.1"))])
        .await
}

pub async fn read_version(reader: &DbReader<MemReadStorage>) -> Result<String, TransactError> {
    let val = reader.get_val(EID_SKYBASE, ATTR_SKYBASE_VERSION).await;
    let version = val.as_str().to_string();
    Ok(version)
}
