use crate::api::version::get_version;
use crate::api::{get_entity_attributes_report, get_entities_report};
use hamt2::db::{Ein, Val};
use leptos::prelude::*;

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
        </header>
        <EntitiesSection/>
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
        <section class="section">
            <h1 class="title is-2">"Browse Entities"</h1>
            <Suspense fallback=|| "Loading...">
                <h2 class="title">"Entities"</h2>
                <label class="label" for="select_entity">"Select an entity"</label>
                {move || entities_report.get().map(|report| view! {
                    <div class="select is-multiple">
                    <select id="select-entity" multiple size=8
                        on:change:target=move |ev| {
                            let value = ev.target().value().parse::<i32>().ok().map(|i| Ein(i));
                            set_active_ein.set(value);
                        }>
                        <For
                            each=move || report.eins.clone()
                            key=|it| it.clone()
                            children=move |it| {
                                let ein = it.to_i32();
                                view! {
                                    <option value={ein}>
                                    {format!("\u{2014}\u{00a0}{ein}\u{00a0}\u{2014}")}
                                    </option>
                                }
                            }
                        />
                    </select>
                    </div>
                })}
            </Suspense>
        </section>
        <Show when=move || {active_ein.get().is_some()}>
            <EntityAttributesSection ein={active_ein.get().unwrap()}/>
        </Show>
    }
}

#[component]
pub fn EntityAttributesSection(ein: Ein) -> impl IntoView {
    let attributes_report = Resource::new(
        || (),
        move |_| async move {
            let report = get_entity_attributes_report(ein)
                .await
                .expect("entity_attributes_report should exist");
            report
        },
    );
    let (active_attr, set_active_attr) = signal(None::<String>);
    view! {
        <section class="section">
            <h2 class="title">{format!("Entity\u{2011}{}", ein.to_i32())}</h2>
            <Suspense fallback=|| "Loading...">
                <p>"Select an attribute"</p>
                {move || Suspend::new(async move {
                    let report = attributes_report.await;
                    view! {
                        <select id="attr-select" size=10
                            on:change:target=move |ev| {
                                let value = ev.target().value();
                                set_active_attr.set(Some(value));
                            }>
                            <For
                                each=move || report.names.clone()
                                key=|name| name.clone()
                                children=move |name| view! {
                                    <option>{name}</option>
                                }
                            />
                        </select>
                    }
                })}
            </Suspense>
        </section>
        <Show when=move || {active_attr.get().is_some()}>
            <ValueSection ein=ein attr={active_attr.get().unwrap()}/>
        </Show>
    }
}

#[component]
pub fn ValueSection(ein: Ein, attr: String) -> impl IntoView {
    let (val, _set_val) = signal(Val::U32(42));
    let (val_type, val_string) = {
        let val = val.get();
        match val {
            Val::U32(v) => ("numeric".to_string(), v.to_string()),
            Val::String(v) => ("string".to_string(), v),
        }
    };
    let ein = ein.to_i32();
    let entity_name =
        format!("\u{301a}\u{00a0}Entity\u{2011}{ein}\u{00a0}\u{301b} \u{2192} {attr}");
    view! {
        <section class="section">
            <h2 class="title">{entity_name}</h2>
            <div>{format!("Value: {}", val_string)}</div>
            <div>{format!("Type: {val_type}")}</div>
        </section>
    }
}
