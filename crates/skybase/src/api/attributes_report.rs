use leptos::prelude::ServerFnError;
use leptos::server;
use serde::{Deserialize, Serialize};
use sky_types::db::Ein;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AttributesReport {
    pub names: Vec<String>,
}

#[server]
pub async fn get_entity_attributes_report(ein: Ein) -> Result<AttributesReport, ServerFnError> {
    use sky_types::db::Attr;
    let attrs: Vec<Attr> = vec![];
    let attr_names = attrs.into_iter().map(|it| it.to_name()).collect::<Vec<_>>();
    Ok(AttributesReport { names: attr_names })
}
