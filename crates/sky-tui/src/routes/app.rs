use ratatui_kit::crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui_kit::prelude::*;
use ratatui_kit::ratatui::prelude::{Constraint, Line, Style};

#[component]
pub fn App(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut exit = hooks.use_exit();
    let items = ["db/ident", "db/cardinality"];
    let mut selected = hooks.use_state(|| None::<String>);
    hooks.use_event_handler(EventScope::Current, EventPriority::Normal, move |event| {
        let Event::Key(key) = event else {
            return EventResult::Ignored;
        };
        if key.kind != KeyEventKind::Press {
            return EventResult::Ignored;
        };
        if matches!(
            (key.code, key.modifiers),
            (KeyCode::Char('q'), KeyModifiers::CONTROL)
        ) {
            exit();
            return EventResult::Consumed;
        }
        EventResult::Ignored
    });

    element!(
        Center(width: Constraint::Length(48), height: Constraint::Length(9)) {
            Select<&'static str>(
                items: items,
                default_index: Some(0),
                highlight_symbol: "> ",
                empty_message: "No attributes",
                on_select: move |item: &'static str| {
                    selected.set(Some(item.to_string()));
                },
            )
            if let Some(selected) = selected.read().clone() {
                Text(
                    text: Line::styled(
                        format!("{}", selected),
                        Style::new().green().bold()
                    ).centered()
                )
            }
        }
    )
}
