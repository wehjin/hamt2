use crate::styles::{border_style, highlight_style};
use ratatui_kit::prelude::*;
use ratatui_kit::ratatui::prelude::{Line, Span, Style};
use sky_types::db::Ein;
use std::collections::HashMap;

#[derive(Default, Props)]
pub struct EinSelectProps {
    pub items: Vec<Ein>,
    pub selected: Option<Ein>,
    pub active: bool,
    pub focused: bool,
    pub on_select: Handler<'static, Ein>,
}

#[component]
pub fn EinSelect(props: &mut EinSelectProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut on_select = props.on_select.take();
    let palette = hooks.use_palette();
    let accent = Style::new().fg(palette.accent);
    let border_style = border_style(palette, props.focused);
    let highlight_style = highlight_style(palette, props.focused);

    let mut items = props.items.clone();
    items.sort();
    items.dedup();

    let lookup = items
        .iter()
        .map(|ein| (ein.0.to_string(), *ein))
        .collect::<HashMap<_, _>>();
    let rows = items
        .iter()
        .map(|ein| {
            Line::from(vec![
                Span::styled("◆ ", accent),
                Span::styled(ein.0.to_string(), Style::new().bold()),
            ])
        })
        .collect::<Vec<_>>();
    let index = props
        .selected
        .and_then(|ein| items.iter().position(|it| it == &ein))
        .or(None);
    element!(
        Select<Line<'static>>(
            items: rows,
            default_index: index,
            active: props.active,
            highlight_symbol: "> ",
            border_style: border_style,
            highlight_style: highlight_style,
            empty_message: "No entities",
            on_select: move |line: Line<'static>| {
                let key = line
                    .spans
                    .last()
                    .map(|span| span.content.to_string())
                    .unwrap_or_default();
                if let Some(ein) = lookup.get(&key) {
                    on_select(*ein);
                }
            },
        )
    )
}
