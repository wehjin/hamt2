use leptos::prelude::ServerFnError;
use leptos::server;
use serde::{Deserialize, Serialize};
use sky_types::db::Ein;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct EntitiesReport {
    pub eins: Vec<Ein>,
}

#[server]
pub async fn get_entities_report() -> Result<EntitiesReport, ServerFnError> {
    use sky_db::find::AllEins;
    use sky_db::handle::DbHandle;
    use sky_db::query::DbQuery;
    use sky_db::storage::MemDbStorage;
    use leptos::prelude::expect_context;

    let reader = expect_context::<DbHandle<MemDbStorage>>()
        .to_reader()
        .await?;
    let eins = reader.find(AllEins).await;
    let report = EntitiesReport { eins };
    Ok(report)
}
