use leptos::prelude::*;

#[server]
pub async fn get_version() -> Result<String, ServerFnError> {
    use sky_types::db::{Ein, Val};
    let binds = vec![(Ein(1), Val::from_str("dummy"))];
    let (_ein, val) = binds.first().unwrap();
    let val = val.as_str();
    Ok(val.to_string())
}
