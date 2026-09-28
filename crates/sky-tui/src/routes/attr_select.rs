use crate::styles::highlight_style;
use ratatui_kit::prelude::*;
use ratatui_kit::ratatui::prelude::Style;
use sky_types::db::Attr;
use std::collections::HashMap;

#[derive(Default, Props)]
pub struct AttrSelectProps {
    pub items: Vec<Attr>,
    pub selected: Option<Attr>,
    pub active: bool,
    pub focused: bool,
    pub on_select: Handler<'static, Attr>,
}

#[component]
pub fn AttrSelect(props: &mut AttrSelectProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut on_select = props.on_select.take();
    let palette = hooks.use_palette();
    let border_style = if props.focused {
        Style::new().fg(palette.border_active)
    } else {
        Style::new().fg(palette.border)
    };
    let highlight_style = highlight_style(palette, props.focused);
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
            border_style: border_style,
            highlight_style: highlight_style,
            empty_message: "No attributes",
            on_select: move |item: String| {
                on_select(lookup[&item].clone());
            },
        )
    )
}
