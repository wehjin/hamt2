use crate::api::version::get_version;
use crate::api::{get_entities_report, get_entity_attributes_report, get_value_report};
use crate::components::TitleAndDelete;
use crate::routes::wss_sandbox::WebSocketSandbox;
use leptos::prelude::*;
use sky_types::db::{Attr, Ein, Val};

#[component]
pub fn HomePage() -> impl IntoView {
    let version = Resource::new(
        move || (),
        |_| async move { get_version().await.expect("version should exist") },
    );
    view! {
        <header class="header">
            <Suspense fallback=|| "Loading...">
                <p class="subtitle">
                    "Skybase version: "
                    {move || Suspend::new(async move { version.await })}
                </p>
            </Suspense>
            <section class="section">
                <WebSocketSandbox/>
            </section>
            <section class="section">
                <h1 class="title is-2">"Browse Entities"</h1>
            </section>
        </header>
        <div class="grid">
            <EntitiesSection/>
        </div>
    }
}

#[derive(Clone)]
pub struct AttributeOptionData {
    pub name: String,
}

#[component]
pub fn EntitiesSection() -> impl IntoView {
    let entities_report = Resource::new(
        || (),
        |_| async move {
            get_entities_report()
                .await
                .expect("entity_report should exist")
        },
    );
    let (active_ein, set_active_ein) = signal(None::<Ein>);
    view! {
        <div class="cell is-flex"><div class="box is-flex-grow-1">
            <Suspense fallback=|| "Loading...">
                <h2 class="title">"🌐 Entities"</h2>
                <label class="label" for="select_entity">"Select an entity"</label>
                {move || entities_report.get().map(|report| view! {
                    <div class="select is-multiple is-fullwidth">
                        <select id="select-entity" multiple size=8
                            on:change:target=move |ev| {
                                let value = ev.target().value().parse::<i32>().ok().map(|i| Ein(i));
                                set_active_ein.set(value);
                            }>
                            <For
                                each=move || report.eins.clone()
                                key=|it| it.clone()
                                children=move |ein| {
                                    let ein_i32 = ein.to_i32();
                                    view! {
                                        <option value={ein_i32}>
                                        {format_ein(ein)}
                                        </option>
                                    }
                                }
                            />
                        </select>
                    </div>
                })}
            </Suspense>
        </div></div>
        <Show when=move || {active_ein.get().is_some()}>
            <AttributesSection ein={active_ein.get().unwrap()}/>
        </Show>
    }
}

#[component]
pub fn AttributesSection(ein: Ein) -> impl IntoView {
    let attributes_report = Resource::new(
        || (),
        move |_| async move {
            let report = get_entity_attributes_report(ein)
                .await
                .expect("entity_attributes_report should exist");
            report
        },
    );
    let (active_attr, set_active_attr) = signal(None::<Attr>);
    let title = format_ein(ein);
    view! {
        <div class="cell is-flex"><div class="box is-flex-grow-1">
            <TitleAndDelete title=title/>
            <Suspense fallback=|| "Loading...">
                <label for="attr-select" class="label">"Select an attribute"</label>
                {move || Suspend::new(async move {
                    let report = attributes_report.await;
                    view! {
                        <div class="select is-multiple is-fullwidth">
                        <select id="attr-select" multiple size=8
                            on:change:target=move |ev| {
                                let value = ev.target().value();
                                set_active_attr.set(Some(Attr::from(value)));
                            }>
                            <For
                                each=move || report.names.clone()
                                key=|name| name.clone()
                                children=move |name| view! {
                                    <option>{name}</option>
                                }
                            />
                        </select>
                        </div>
                    }
                })}
            </Suspense>
        </div></div>
        <Show when=move || {active_attr.get().is_some()}>
            <ValueSection ein=ein attr={active_attr.get().unwrap()}/>
        </Show>
    }
}

fn format_ein(ein: Ein) -> String {
    format!("⌗{}", ein.to_i32())
}

#[component]
pub fn ValueSection(ein: Ein, attr: Attr) -> impl IntoView {
    let value_report = {
        let attr = attr.clone();
        Resource::new(
            || (),
            move |_| {
                let attr = attr.clone();
                async move {
                    get_value_report(ein, attr)
                        .await
                        .expect("value_report should exist")
                }
            },
        )
    };
    let title = format!("⌖\u{202F}{attr}");
    view! {
        <div class="cell is-flex"><div class="box is-flex-grow-1">
            <TitleAndDelete title=title/>
            <Suspense fallback=|| "">
                {move || Suspend::new(async move {
                    let value_report = value_report.await;
                    match value_report.val {
                        Some(val) => {
                            let (val_type, val_string) = match val {
                                Val::U32(v) => ("numeric".to_string(), v.to_string()),
                                Val::String(v) => ("string".to_string(), v.to_string()),
                            };
                            view! {
                                <label for="value-show" class="label">"Value"</label>
                                <div class="field has-addons">
                                    <div class="control">
                                        <a class="button is-static">{{val_type}}</a>
                                    </div>
                                    <div class="control is-expanded">
                                        <input id="value-show" class="input" type="text" value=val_string readonly/>
                                    </div>
                                </div>
                            }.into_any()
                        }
                        None => {
                            view! { <p>"No value found!"</p>}.into_any()
                        }
                    }
                })}
            </Suspense>
        </div></div>
    }
}
