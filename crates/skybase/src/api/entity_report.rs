use hamt2::db::Ein;
use leptos::prelude::ServerFnError;
use leptos::server;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct EntitiesReport {
    pub eins: Vec<Ein>,
}

#[server]
pub async fn get_entities_report() -> Result<EntitiesReport, ServerFnError> {
    use hamt2::db::handle::DbHandle;
    use hamt2::db::query::DbQuery;
    use hamt2::find::AllEins;
    use hamt2::trie::base_storage::mem::MemBaseStorage;
    use leptos::prelude::expect_context;

    let reader = expect_context::<DbHandle<MemBaseStorage>>()
        .to_reader()
        .await?;
    let eins = reader.find(AllEins).await?;
    let report = EntitiesReport { eins };
    Ok(report)
}
