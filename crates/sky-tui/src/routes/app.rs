use ratatui_kit::crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui_kit::prelude::*;
use ratatui_kit::ratatui::prelude::{Constraint, Line, Style};
use sky_types::db;
use sky_types::db::Attr;

#[derive(Debug, Clone)]
pub struct AttrList(Vec<Attr>);

impl AttrList {
    pub fn new() -> Self {
        let vec = vec![db::ident(), db::cardinality(), db::query()];
        AttrList(vec)
    }
    pub fn to_strings(&self) -> Vec<String> {
        self.0.iter().map(|a| a.to_string()).collect()
    }
}

#[component]
pub fn App(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut exit = hooks.use_exit();
    let mut active_attr = hooks.use_state(|| None::<Attr>);
    let attrs = AttrList::new();
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
                Select<String>(
                    items: attrs.to_strings(),
                    default_index: Some(0),
                    highlight_symbol: "> ",
                    empty_message: "No attributes",
                    on_select: move |item: String| {
                        active_attr.set(Some(Attr(item)));
                    },
                )
            }
        }
    )
}
