use hamt2::trie::base_storage::mem::MemBaseStorage;
use leptos::prelude::*;
use skydb::SkyViewer;

#[component]
pub fn HomePage(storage: MemBaseStorage) -> impl IntoView {
    let viewer = SkyViewer::start(storage);
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;
    view! {
        <h1>"Welcome to Skybase!"</h1>
        <p>"version: " {move || viewer.get_version()}</p>
        <button on:click=on_click>"Click Me: " {count}</button>
    }
}
