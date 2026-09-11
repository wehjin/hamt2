#[cfg(feature = "ssr")]
use hamt2::db::handle::DbHandle;
use leptos::prelude::*;
#[cfg(feature = "ssr")]
use skydb::read_version;
#[cfg(feature = "ssr")]
use hamt2::trie::base_storage::mem::MemBaseStorage;

#[server]
pub async fn get_version() -> Result<String, ServerFnError> {
    let db = expect_context::<DbHandle<MemBaseStorage>>();
    let reader = db.to_reader().await?;
    let version = read_version(&reader).await?;
    Ok(version)
}
