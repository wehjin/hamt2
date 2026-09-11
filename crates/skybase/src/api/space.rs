use hamt2::trie::base_storage::mem::MemBaseStorage;
use leptos::prelude::*;

#[server]
pub async fn get_storage() -> Result<MemBaseStorage, ServerFnError> {
    let db = skydb::start_db().await?;
    let storage = db.close();
    Ok(storage)
}
