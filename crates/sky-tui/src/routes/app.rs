use ratatui_kit::crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui_kit::prelude::*;
use ratatui_kit::ratatui::prelude::{Constraint, Line, Style};

#[component]
pub fn App(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut message = hooks.use_state(|| "Hello".to_string());

    hooks.use_event_handler(EventScope::Current, EventPriority::Normal, move |event| {
        let Event::Key(key) = event else {
            return EventResult::Ignored;
        };
        if key.kind == KeyEventKind::Press {
            return match key.code {
                KeyCode::Char(' ') if key.modifiers == KeyModifiers::CONTROL => {
                    message.set("World!".to_string());
                    EventResult::Consumed
                }
                KeyCode::Esc if key.modifiers == KeyModifiers::NONE => {
                    message.set("Hello,".to_string());
                    EventResult::Consumed
                }
                _ => EventResult::Ignored,
            };
        }
        EventResult::Ignored
    });

    element!(
        Center(width: Constraint::Length(48), height: Constraint::Length(9)) {
            Text(text: Line::styled(
                format!("{}", message.read().to_string()),
                Style::new().green().bold(),
            ).centered())
        }
    )
}
