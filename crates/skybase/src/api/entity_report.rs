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
    use hamt2::handle::DbHandle;
    use hamt2::query::DbQuery;
    use hamt2::find::AllEins;
    use hamt2::storage::MemDbStorage;
    use leptos::prelude::expect_context;

    let reader = expect_context::<DbHandle<MemDbStorage>>()
        .to_reader()
        .await?;
    let eins = reader.find(AllEins).await?;
    let report = EntitiesReport { eins };
    Ok(report)
}
