use sky_types::db::Ein;
use leptos::prelude::ServerFnError;
use leptos::server;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AttributesReport {
    pub names: Vec<String>,
}

#[server]
pub async fn get_entity_attributes_report(ein: Ein) -> Result<AttributesReport, ServerFnError> {
    use sky_db::find::AttrsOfEin;
    use sky_db::handle::DbHandle;
    use sky_db::query::DbQuery;
    use sky_trie::storage::mem::MemStorage;
    use leptos::prelude::expect_context;

    let reader = expect_context::<DbHandle<MemStorage>>()
        .to_reader()
        .await?;
    let attrs = reader.find(AttrsOfEin::new(ein)).await;
    let attr_names = attrs.into_iter().map(|it| it.to_name()).collect::<Vec<_>>();
    Ok(AttributesReport { names: attr_names })
}
