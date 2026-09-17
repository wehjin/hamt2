use leptos::prelude::ServerFnError;
use leptos::server;
use serde::{Deserialize, Serialize};
use sky_types::db::{Attr, Ein, Val};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ValueReport {
    pub val: Option<Val>,
}

#[server]
pub async fn get_value_report(ein: Ein, attr: Attr) -> Result<ValueReport, ServerFnError> {
    let value: Vec<Val> = vec![];
    let Some(val) = value.first() else {
        return Ok(ValueReport { val: None });
    };

    let report = ValueReport {
        val: Some(val.clone()),
    };
    Ok(report)
}
