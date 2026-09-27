use crate::routes::attr_select::AttrSelect;
use ratatui_kit::crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui_kit::prelude::*;
use ratatui_kit::ratatui::prelude::{Constraint, Line, Style};
use sky_types::db;
use sky_types::db::Attr;

#[component]
pub fn App(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut exit = hooks.use_exit();
    let mut active_attr = hooks.use_state(|| None::<Attr>);
    hooks.use_event_handler(EventScope::Current, EventPriority::Normal, move |event| {
        let Event::Key(key) = event else {
            return EventResult::Ignored;
        };
        if key.kind != KeyEventKind::Press {
            return EventResult::Ignored;
        };
        if matches!(
            (key.code, key.modifiers),
            (KeyCode::Char('w'), KeyModifiers::CONTROL)
        ) {
            active_attr.set(None);
        } else if matches!(
            (key.code, key.modifiers),
            (KeyCode::Char('q'), KeyModifiers::CONTROL)
        ) {
            exit();
            return EventResult::Consumed;
        }
        EventResult::Ignored
    });
    let attrs = vec![db::ident(), db::cardinality(), db::query()];
    element!(
        Center(width: Constraint::Length(48), height: Constraint::Length(9)) {
            if let Some(active_attr) = active_attr.read().clone() {
                Text(
                    text: Line::styled(
                        format!("{}", active_attr),
                        Style::new().green().bold()
                    ).centered()
                )
            } else {
                AttrSelect(
                    items: attrs.clone(),
                    selected: active_attr.read().clone(),
                    on_select: move |it| {
                        active_attr.set(Some(it));
                    }
                )
            }
        }
    )
}
