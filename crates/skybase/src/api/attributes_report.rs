use hamt2::db::Ein;
use leptos::prelude::ServerFnError;
use leptos::server;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AttributesReport {
    pub names: Vec<String>,
}

#[server]
pub async fn get_entity_attributes_report(ein: Ein) -> Result<AttributesReport, ServerFnError> {
    use hamt2::db::handle::DbHandle;
    use hamt2::db_query::DbQuery;
    use hamt2::find::AttrsOfEin;
    use hamt2::trie::prelude::MemTrieStorage;
    use leptos::prelude::expect_context;

    let reader = expect_context::<DbHandle<MemTrieStorage>>()
        .to_reader()
        .await?;
    let attrs = reader.find(AttrsOfEin::new(ein)).await?;
    let attr_names = attrs.into_iter().map(|it| it.to_name()).collect::<Vec<_>>();
    Ok(AttributesReport { names: attr_names })
}
