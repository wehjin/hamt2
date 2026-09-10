use hamt2::space::mem::MemSpace;
use leptos::prelude::*;

#[server]
pub async fn get_space() -> Result<MemSpace, ServerFnError> {
    let db = skydb::start_db().await?;
    let space = db.to_space();
    Ok(space)
}
