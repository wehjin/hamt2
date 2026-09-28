use crate::components::loading::Loading;
use crate::routes::app::DB_VIEW;
use crate::routes::attr_select::AttrSelect;
use crate::routes::ein_select::EinSelect;
use ratatui_kit::crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui_kit::prelude::*;
use ratatui_kit::ratatui::layout::{Constraint, Direction};
use ratatui_kit::ratatui::style::Style;
use ratatui_kit::ratatui::text::{Line, Span};
use sky_db::find::{AllAttrs, EinsWithAttr};
use sky_db::reader::DbReader;
use sky_db::traits::DbQuery;
use sky_types::db::{Attr, Ein};
use sky_types::storage::MemView;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Layout {
    OneColumn,
    TwoColumns,
    ThreeColumns,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Focus {
    Val,
    Ein,
    Attr,
}

fn print_ein<'a>(ein: Ein, palette: Palette, margins: bool) -> Line<'a> {
    let mut spans = vec![
        Span::styled("◆ ", Style::new().fg(palette.accent)),
        Span::styled(ein.0.to_string(), Style::new().bold()),
    ];
    if margins {
        spans.insert(0, Span::styled(" ", Style::new().bold()));
        spans.push(Span::styled(" ", Style::new().bold()));
    }
    Line::from(spans)
}

async fn attrs_in_view(opt_view: &Option<DbReader<MemView>>) -> Option<Vec<Attr>> {
    if let Some(view) = opt_view {
        let view_attrs = view.find(AllAttrs).await;
        Some(view_attrs)
    } else {
        None
    }
}

async fn eins_with_attr_in_view(
    opt_attr: &Option<Attr>,
    opt_view: &Option<DbReader<MemView>>,
) -> Option<Vec<Ein>> {
    if let (Some(view), Some(attr)) = (opt_view, opt_attr) {
        let view_eins = view.find(EinsWithAttr::new(attr.clone())).await;
        Some(view_eins)
    } else {
        None
    }
}

#[component]
pub fn Browser(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let db_view = hooks.use_atom(&DB_VIEW);
    let mut attrs = hooks.use_state(|| None::<Vec<Attr>>);
    let mut active_attr = hooks.use_state(|| None::<Attr>);
    let mut eins = hooks.use_state(|| None::<Vec<Ein>>);
    let mut active_ein = hooks.use_state(|| None::<Ein>);

    let palette = hooks.use_palette();
    let mut focus = hooks.use_state(|| Focus::Attr);
    let mut layout = hooks.use_state(|| Layout::OneColumn);

    let attrs_db = db_view.read().clone();
    let attrs_deps = attrs_db.clone();
    hooks.use_async_effect(
        async move {
            let view_attrs = attrs_in_view(&attrs_db).await;
            attrs.set(view_attrs);
        },
        attrs_deps,
    );

    let eins_attr = active_attr.read().clone();
    let eins_db = db_view.read().clone();
    let eins_deps = (eins_attr.clone(), eins_db.clone());
    hooks.use_async_effect(
        async move {
            let view_eins = eins_with_attr_in_view(&eins_attr, &eins_db).await;
            eins.set(view_eins);
        },
        eins_deps,
    );

    hooks.use_event_handler(EventScope::Current, EventPriority::Normal, move |event| {
        let Event::Key(key) = event else {
            return EventResult::Ignored;
        };
        if key.kind != KeyEventKind::Press {
            return EventResult::Ignored;
        };
        if matches!(key.code, KeyCode::Char('h') | KeyCode::Left) {
            match focus.get() {
                Focus::Val => (),
                Focus::Ein => match layout.get() {
                    Layout::OneColumn | Layout::TwoColumns => (),
                    Layout::ThreeColumns => focus.set(Focus::Val),
                },
                Focus::Attr => match layout.get() {
                    Layout::OneColumn => (),
                    Layout::TwoColumns | Layout::ThreeColumns => focus.set(Focus::Ein),
                },
            }
            return EventResult::Consumed;
        } else if matches!(key.code, KeyCode::Char('l') | KeyCode::Right) {
            match focus.get() {
                Focus::Val => focus.set(Focus::Ein),
                Focus::Ein => focus.set(Focus::Attr),
                Focus::Attr => (),
            }
            return EventResult::Consumed;
        } else if matches!(
            (key.code, key.modifiers),
            (KeyCode::Char('w'), KeyModifiers::CONTROL)
        ) {
            if active_ein.read().is_some() {
                active_ein.set(None);
                layout.set(Layout::TwoColumns);
                if focus.get() == Focus::Val {
                    focus.set(Focus::Ein);
                }
            } else {
                active_attr.set(None);
                layout.set(Layout::OneColumn);
                match focus.get() {
                    Focus::Val | Focus::Ein => {
                        focus.set(Focus::Attr);
                    }
                    Focus::Attr => (),
                }
            }
            return EventResult::Consumed;
        }
        EventResult::Ignored
    });
    element!(
        View {
            View(flex_direction: Direction::Horizontal, gap: 1) {
                View(width: match *layout.read() {
                    Layout::OneColumn => Constraint::Percentage(0),
                    Layout::TwoColumns => Constraint::Percentage(0),
                    Layout::ThreeColumns => Constraint::Percentage(50),
                }) {
                    if let Some(ein) = active_ein.get() {
                         Border(
                                width: Constraint::Fill(1),
                                top_title: print_ein(ein, palette, true).centered(),
                        ) {
                            Loading()
                        }
                    } else {
                        Text(text: "loading".to_string())
                    }
                }
                View(width: match *layout.read() {
                    Layout::OneColumn => Constraint::Percentage(0),
                    Layout::TwoColumns => Constraint::Percentage(70),
                    Layout::ThreeColumns => Constraint::Percentage(20),
                }) {
                    if let Some(eins) = eins.read().clone() {
                        EinSelect(
                            items: eins,
                            selected: *active_ein.read(),
                            active: *focus.read() == Focus::Ein,
                            focused: *focus.read() == Focus::Ein,
                            on_select: move |it| {
                                active_ein.set(Some(it));
                                layout.set(Layout::ThreeColumns);
                            },
                        )
                    } else {
                        Loading()
                    }
                }
                View(width: match *layout.read() {
                    Layout::OneColumn => Constraint::Percentage(100),
                    Layout::TwoColumns => Constraint::Percentage(30),
                    Layout::ThreeColumns => Constraint::Percentage(30),
                }) {
                    if let Some(attrs) = attrs.read().clone() {
                        AttrSelect(
                            items: attrs,
                            selected: active_attr.read().clone(),
                            active: *focus.read() == Focus::Attr,
                            focused: *focus.read() == Focus::Attr,
                            on_select: move |it| {
                                active_attr.set(Some(it));
                                active_ein.set(None);
                                layout.set(Layout::TwoColumns);
                            }
                        )
                    } else {
                        Loading()
                    }
                }
            }
        }
    )
}
