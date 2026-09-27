use ratatui_kit::prelude::*;
use sky_types::db::Attr;
use std::collections::HashMap;

#[derive(Default, Props)]
pub struct AttrSelectProps {
    pub items: Vec<Attr>,
    pub selected: Option<Attr>,
    pub active: bool,
    pub on_select: Handler<'static, Attr>,
}

#[component]
pub fn AttrSelect(props: &mut AttrSelectProps) -> impl Into<AnyElement<'static>> {
    let mut on_select = props.on_select.take();
    let lookup = props
        .items
        .iter()
        .map(|a| (a.to_string(), a.clone()))
        .collect::<HashMap<_, _>>();
    let mut items = lookup.keys().map(|it| it.to_string()).collect::<Vec<_>>();
    items.sort();
    let index = if let Some(attr) = &props.selected {
        let string = attr.to_string();
        items.iter().position(|a| a == &string)
    } else {
        Some(0)
    };
    element!(
        Select<String>(
            items: items,
            default_index: index,
            active: props.active,
            highlight_symbol: "> ",
            empty_message: "No attributes",
            on_select: move |item: String| {
                on_select(lookup[&item].clone());
            },
        )
    )
}
