use crate::routes::browser::Browser;
use ratatui_kit::crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui_kit::prelude::*;

#[component]
pub fn App(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut exit = hooks.use_exit();
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
    element!(Browser {})
}
