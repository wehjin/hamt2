use crate::routes::attr_select::AttrSelect;
use crate::routes::ein_select::EinSelect;
use ratatui_kit::crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui_kit::prelude::*;
use ratatui_kit::ratatui::layout::Direction;
use ratatui_kit::ratatui::prelude::{Constraint, Line, Span, Style};
use sky_types::db;
use sky_types::db::{Attr, Ein, ein};

#[derive(Copy, Clone, PartialEq, Eq)]
enum Focus {
    Ein,
    Attr,
}

#[component]
pub fn App(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut exit = hooks.use_exit();
    let palette = hooks.use_palette();
    let mut focus = hooks.use_state(|| Focus::Attr);
    let mut active_attr = hooks.use_state(|| None::<Attr>);
    let mut active_ein = hooks.use_state(|| None::<Ein>);
    let eins_state = hooks.use_state(|| Some(vec![ein(3), ein(5), ein(8), ein(13), ein(21)]));

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
            if active_ein.read().is_some() {
                active_ein.set(None);
            } else {
                active_attr.set(None);
                focus.set(Focus::Attr);
            }
            return EventResult::Consumed;
        } else if matches!(
            (key.code, key.modifiers),
            (KeyCode::Char('q'), KeyModifiers::CONTROL)
        ) {
            exit();
            return EventResult::Consumed;
        } else if matches!(key.code, KeyCode::Char('h') | KeyCode::Left) {
            focus.set(Focus::Ein);
            return EventResult::Consumed;
        } else if matches!(key.code, KeyCode::Char('l') | KeyCode::Right) {
            focus.set(Focus::Attr);
            return EventResult::Consumed;
        }
        EventResult::Ignored
    });
    let attrs = vec![db::ident(), db::cardinality(), db::query()];
    element!(
        View {
            if let Some(attr) = active_attr.read().clone() {
                View(flex_direction: Direction::Horizontal, gap: 2) {
                    Border(
                        width: Constraint::Fill(1),
                        top_title: Line::from(format!(" [{}] ← Entities ", attr)).centered(),
                    ) {
                        if let Some(ein) = active_ein.read().clone() {
                            View(flex_direction: Direction::Horizontal, gap: 1) {
                                View(width: Constraint::Fill(1)) {
                                    Center(
                                        width: Constraint::Length(48),
                                        height: Constraint::Length(9),
                                    ) {
                                        Text(
                                            text: Line::from(vec![
                                                Span::styled(
                                                    "◆ ",
                                                    Style::new().fg(palette.accent),
                                                ),
                                                Span::styled(
                                                    ein.0.to_string(),
                                                    Style::new().bold(),
                                                ),
                                            ])
                                            .centered(),
                                        )
                                    }
                                }
                                View(width: Constraint::Percentage(20)) {
                                    if let Some(eins) = eins_state.read().clone() {
                                        EinSelect(
                                            items: eins,
                                            selected: Some(ein.clone()),
                                            active: *focus.read() == Focus::Ein,
                                            on_select: move |it| {
                                                active_ein.set(Some(it));
                                            },
                                        )
                                    } else {
                                        Text(
                                            text: Line::styled(
                                                "loading".to_string(),
                                                Style::new().green().bold(),
                                            )
                                            .centered(),
                                        )
                                    }
                                }
                            }
                        } else {
                            Center(
                                width: Constraint::Length(48),
                                height: Constraint::Length(9),
                            ) {
                                if let Some(eins) = eins_state.read().clone() {
                                    // Have eins data; zero eins are handled by the
                                    // EinSelect's empty_message.
                                    EinSelect(
                                        items: eins,
                                        selected: active_ein.read().clone(),
                                        active: *focus.read() == Focus::Ein,
                                        on_select: move |it| {
                                            active_ein.set(Some(it));
                                        },
                                    )
                                } else {
                                    // Loading eins data.
                                    Text(
                                        text: Line::styled(
                                            "loading".to_string(),
                                            Style::new().green().bold(),
                                        )
                                        .centered(),
                                    )
                                }
                            }
                        }
                    }
                    View(width: Constraint::Percentage(30)) {
                        AttrSelect(
                            items: attrs.clone(),
                            selected: Some(attr.clone()),
                            active: *focus.read() == Focus::Attr,
                            on_select: move |it| {
                                active_attr.set(Some(it));
                                active_ein.set(None);
                                focus.set(Focus::Ein);
                            }
                        )
                    }
                }
            } else {
                Center(width: Constraint::Length(48), height: Constraint::Length(9)) {
                    AttrSelect(
                        items: attrs.clone(),
                        selected: active_attr.read().clone(),
                        active: *focus.read() == Focus::Attr,
                        on_select: move |it| {
                            active_attr.set(Some(it));
                            active_ein.set(None);
                            focus.set(Focus::Ein);
                        }
                    )
                }
            }
        }
    )
}
