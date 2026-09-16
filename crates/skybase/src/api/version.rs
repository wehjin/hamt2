use leptos::prelude::*;
#[server]
pub async fn get_version() -> Result<String, ServerFnError> {
    use crate::db::skybase_version;
    use sky_db::find::BindsForAttr;
    use sky_db::handle::DbHandle;
    use sky_db::query::DbQuery;
    use sky_trie::storage::mem::MemStorage;
    let db = expect_context::<DbHandle<MemStorage>>();
    let reader = db.to_reader().await?;
    let binds = reader.find(BindsForAttr::new(skybase_version())).await;
    let (_ein, val) = binds.first().unwrap();
    let val = val.as_str();
    Ok(val.to_string())
}
