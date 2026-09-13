use leptos::prelude::*;
#[server]
pub async fn get_version() -> Result<String, ServerFnError> {
    use hamt2::db::handle::DbHandle;
    use hamt2::db_query::DbQuery;
    use hamt2::find::BindsForAttr;
    use hamt2::trie::prelude::MemTrieStorage;
    use crate::db::ATTR_SKYBASE_VERSION;
    let db = expect_context::<DbHandle<MemTrieStorage>>();
    let reader = db.to_reader().await?;
    let binds = reader.find(BindsForAttr::new(ATTR_SKYBASE_VERSION)).await?;
    let (_ein, val) = binds.first().unwrap();
    let val = val.as_str();
    Ok(val.to_string())
}
