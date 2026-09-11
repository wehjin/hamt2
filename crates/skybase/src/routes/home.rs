use crate::api::browser::get_version;
use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView {
    let version = Resource::new(
        move || (),
        |_| async move { get_version().await.expect("version should exist") },
    );
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;
    view! {
        <h1>"Welcome to Skybase!"</h1>
        <Suspense fallback=|| "Loading...">
            <p>"version: "
                {
                    let version = version.clone();
                    move || Suspend::new(async move { version.await })
                }
            </p>
        </Suspense>
        <button on:click=on_click>"Click Me: " {count}</button>
    }
}
