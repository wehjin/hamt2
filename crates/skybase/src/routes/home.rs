use hamt2::space::mem::MemSpace;
use leptos::prelude::*;
use skydb::SkyViewer;

#[component]
pub fn HomePage(space: MemSpace) -> impl IntoView {
    let viewer = SkyViewer::start(space);
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;
    view! {
        <h1>"Welcome to Skybase!"</h1>
        <p>"version: " {move || viewer.get_version()}</p>
        <button on:click=on_click>"Click Me: " {count}</button>
    }
}
