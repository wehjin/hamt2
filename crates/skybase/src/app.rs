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

    view! {
        <Title text="Skybase"/>
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

#[server]
pub async fn get_version() -> Result<String, ServerFnError> {
    let db_state = expect_context::<crate::state::DbState>();
    let version = db_state.skybase_version.clone();
    Ok(version)
}

#[component]
fn HomePage() -> impl IntoView {
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;

    let version = Resource::new(
        || (),
        |_| async move { get_version().await.unwrap_or_default() },
    );

    view! {
        <h1>"Welcome to Skybase!"</h1>
        <Suspense fallback=|| "Loading...".into_view()>
            <p>"version: " {move || version.get().unwrap_or_default()}</p>
        </Suspense>
        <button on:click=on_click>"Click Me: " {count}</button>
    }
}
