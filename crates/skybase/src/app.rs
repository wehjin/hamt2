use crate::api::space::get_storage;
use crate::routes::home::HomePage;
use leptos::prelude::*;
use leptos_meta::{MetaTags, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let storage =
        LocalResource::new(|| async move { get_storage().await.expect("getting storage failed") });

    let home_suspense = move || match storage.get() {
        None => view! { <div>"Loading storage…"</div>}.into_any(),
        Some(storage) => view! { <HomePage storage=storage/> }.into_any(),
    };

    view! {
        <Title text="Skybase"/>
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=home_suspense />
                </Routes>
            </main>
        </Router>
    }
}
