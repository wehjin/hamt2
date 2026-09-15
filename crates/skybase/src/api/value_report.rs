use hamt2::db::AttrName;
use leptos::prelude::ServerFnError;
use leptos::server;
use serde::{Deserialize, Serialize};
use sky_types::db::{Ein, Val};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ValueReport {
    pub val: Option<Val>,
}

#[server]
pub async fn get_value_report(ein: Ein, attr_name: AttrName) -> Result<ValueReport, ServerFnError> {
    use hamt2::find::*;
    use hamt2::handle::DbHandle;
    use hamt2::query::DbQuery;
    use hamt2::storage::MemDbStorage;
    use leptos::prelude::expect_context;
    let reader = expect_context::<DbHandle<MemDbStorage>>()
        .to_reader()
        .await?;

    let attr = reader.find(AttrWithName::new(attr_name)).await;
    let Some(attr) = attr.first() else {
        return Ok(ValueReport { val: None });
    };

    let value = reader.find(ValsInSlot::new(ein, *attr)).await;
    let Some(val) = value.first() else {
        return Ok(ValueReport { val: None });
    };

    let report = ValueReport {
        val: Some(val.clone()),
    };
    Ok(report)
}
