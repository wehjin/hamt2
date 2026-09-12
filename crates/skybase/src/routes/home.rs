use crate::api::browser::{get_entity_report, get_version};
use hamt2::db::Val;
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
        <EntAttrValSection/>
    }
}

#[derive(Clone)]
pub struct AttributeOptionData {
    pub name: String,
}

#[component]
pub fn EntAttrValSection() -> impl IntoView {
    let entities_report = Resource::new(
        || (),
        |_| async move {
            get_entity_report()
                .await
                .expect("entity_report should exist")
        },
    );
    let (active_ein, set_active_ein) = signal(None::<i32>);
    view! {
        <section>
            <h1>"Browse Entities"</h1>
            <Suspense fallback=|| "Loading...">
                <h2>"Entities"</h2>
                <p>"Select an entity"</p>
                {move || entities_report.get().map(|report| view! {
                    <select id="ent-select" size=10
                        on:change:target=move |ev| {
                            let value = ev.target().value().parse::<i32>().ok();
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
            <AttrValSection eid={active_ein.get().unwrap()}/>
        </Show>
    }
}

#[component]
pub fn AttrValSection(eid: i32) -> impl IntoView {
    let (active_attr, set_active_attr) = signal(None::<String>);
    let (options, _set_options) = signal(vec![
        AttributeOptionData {
            name: "Counter/count".into(),
        },
        AttributeOptionData {
            name: "Animal/claws".into(),
        },
    ]);
    view! {
        <section>
            <h2>{format!("Entity\u{2011}{eid}")}</h2>
            <p>"Select an attribute"</p>
            <select id="attr-select" size=10
                on:change:target=move |ev| {
                    let value = ev.target().value();
                    set_active_attr.set(Some(value));
                }>
                <For
                    each=move || options.get()
                    key=|option| option.name.clone()
                    children=move |option| view! {<AttributeOption data=option/>}
                />
            </select>
        </section>
        <Show when=move || {active_attr.get().is_some()}>
            <ValSection eid=eid attr={active_attr.get().unwrap()}/>
        </Show>
    }
}

#[component]
pub fn ValSection(eid: i32, attr: String) -> impl IntoView {
    let (val, _set_val) = signal(Val::U32(42));
    let (val_type, val_string) = {
        let val = val.get();
        match val {
            Val::U32(v) => ("numeric".to_string(), v.to_string()),
            Val::String(v) => ("string".to_string(), v),
        }
    };
    view! {
        <section>
            <h2>{format!("Entity\u{2011}{eid}\u{00a0}[\u{00a0}{attr}\u{00a0}]")}</h2>
            <div>{format!("Value: {}", val_string)}</div>
            <div>{format!("Type: {val_type}")}</div>
        </section>
    }
}

#[component]
pub fn AttributeOption(data: AttributeOptionData) -> impl IntoView {
    let AttributeOptionData { name } = data;
    view! {
        <option>{name}</option>
    }
}
