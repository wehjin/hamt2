use crate::api::browser::{get_entity_attributes_report, get_entity_report, get_version};
use hamt2::db::{Ein, Val};
use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView {
    let version = Resource::new(
        move || (),
        |_| async move { get_version().await.expect("version should exist") },
    );
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
            get_entity_report()
                .await
                .expect("entity_report should exist")
        },
    );
    let (active_ein, set_active_ein) = signal(None::<Ein>);
    view! {
        <section>
            <h1>"Browse Entities"</h1>
            <Suspense fallback=|| "Loading...">
                <h2>"Entities"</h2>
                <p>"Select an entity"</p>
                {move || entities_report.get().map(|report| view! {
                    <select id="ent-select" size=10
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
        <section>
            <h2>{format!("Entity\u{2011}{}", ein.to_i32())}</h2>
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
    let entity_name = format!("Entity\u{2011}{ein}\u{00a0}[\u{00a0}{attr}\u{00a0}]");
    view! {
        <section>
            <h2>{entity_name}</h2>
            <div>{format!("Value: {}", val_string)}</div>
            <div>{format!("Type: {val_type}")}</div>
        </section>
    }
}
