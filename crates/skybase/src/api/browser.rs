use hamt2::db::Ein;
#[cfg(feature = "ssr")]
use hamt2::db::handle::DbHandle;
#[cfg(feature = "ssr")]
use hamt2::trie::base_storage::mem::MemBaseStorage;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use skydb::read_version;

#[server]
pub async fn get_version() -> Result<String, ServerFnError> {
    let db = expect_context::<DbHandle<MemBaseStorage>>();
    let reader = db.to_reader().await?;
    let version = read_version(&reader).await?;
    Ok(version)
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct EntityReport {
    pub eins: Vec<Ein>,
}

#[server]
pub async fn get_entity_report() -> Result<EntityReport, ServerFnError> {
    let reader = expect_context::<DbHandle<MemBaseStorage>>()
        .to_reader()
        .await?;
    let eins = reader.list_entities().await;
    let report = EntityReport { eins };
    Ok(report)
}
