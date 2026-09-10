use hamt2::space::mem::MemSpace;
use leptos::prelude::*;
use leptos_meta::{MetaTags, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};
use skydb::SkyViewer;

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

#[server]
pub async fn get_space() -> Result<MemSpace, ServerFnError> {
    let db = skydb::start_db().await?;
    let space = db.to_space();
    Ok(space)
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let space =
        LocalResource::new(|| async move { get_space().await.expect("getting space failed") });

    let home_suspense = move || match space.get() {
        None => view! { <div>"Loading space…"</div>}.into_any(),
        Some(space) => view! { <HomePage space=space/> }.into_any(),
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

#[component]
fn HomePage(space: MemSpace) -> impl IntoView {
    let viewer = SkyViewer::start(space);
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;
    view! {
        <h1>"Welcome to Skybase!"</h1>
        <p>"version: " {move || viewer.get_version()}</p>
        <button on:click=on_click>"Click Me: " {count}</button>
    }
}
