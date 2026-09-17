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
    let eins = vec![];
    let report = EntitiesReport { eins };
    Ok(report)
}
