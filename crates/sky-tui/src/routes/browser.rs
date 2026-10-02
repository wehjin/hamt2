use crate::components::loading::Loading;
use crate::lines::{print_attr_title, print_ein, print_fill};
use crate::routes::app::DB_VIEW;
use crate::routes::attr_select::AttrSelect;
use crate::routes::ein_select::EinSelect;
use crate::styles::border_style;
use ratatui_kit::crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui_kit::prelude::*;
use ratatui_kit::ratatui::layout::{Constraint, Direction};
use ratatui_kit::ratatui::widgets::Block;
use sky_db::DbQuery;
use sky_db::DbReader;
use sky_db::find::{AllAttrs, EinsWithAttr, EntityFills};
use sky_db::trie::MemView;
use sky_db::{Attr, Ein, Fill};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Layout {
    OneColumn,
    TwoColumns,
    ThreeColumns,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Focus {
    Val,
    Ein,
    Attr,
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
    opt_view: &Option<DbReader<MemView>>,
    opt_attr: &Option<Attr>,
) -> Option<Vec<Ein>> {
    if let (Some(view), Some(attr)) = (opt_view, opt_attr) {
        let view_eins = view.find(EinsWithAttr::new(attr.clone())).await;
        Some(view_eins)
    } else {
        None
    }
}

async fn fills_of_ein_in_view(
    opt_view: &Option<DbReader<MemView>>,
    opt_ein: &Option<Ein>,
) -> Option<Vec<Fill>> {
    if let (Some(view), Some(ein)) = (opt_view, opt_ein) {
        let view_fills = view.find(EntityFills(*ein)).await;
        Some(view_fills)
    } else {
        None
    }
}

#[component]
pub fn Browser(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let db_view = hooks.use_atom(&DB_VIEW);
    let mut attrs = hooks.use_state(|| None::<Vec<Attr>>);
    let mut eins = hooks.use_state(|| None::<Vec<Ein>>);
    let mut fills = hooks.use_state(|| None::<Vec<Fill>>);
    let mut active_attr = hooks.use_state(|| None::<Attr>);
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

    let eins_db = db_view.read().clone();
    let eins_attr = active_attr.read().clone();
    let eins_deps = (eins_db.clone(), eins_attr.clone());
    hooks.use_async_effect(
        async move {
            let view_eins = eins_with_attr_in_view(&eins_db, &eins_attr).await;
            let (next_active, next_layout) = match &view_eins {
                Some(eins) if eins.len() == 1 => (Some(eins[0].clone()), Layout::ThreeColumns),
                Some(_eins) => (None, Layout::TwoColumns),
                _ => (None, Layout::OneColumn),
            };
            eins.set(view_eins);
            active_ein.set(next_active);
            layout.set(next_layout);
        },
        eins_deps,
    );

    let fills_db = db_view.read().clone();
    let fills_ein = active_ein.read().clone();
    let fills_deps = (fills_db.clone(), fills_ein.clone());
    hooks.use_async_effect(
        async move {
            let view_fills = fills_of_ein_in_view(&fills_db, &fills_ein).await;
            fills.set(view_fills);
        },
        fills_deps,
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
                    if let (Some(fills),Some(ein)) = (fills.read().clone(), active_ein.read().clone()) {
                        ScrollView(
                            flex_direction: Direction::Vertical,
                            active: *focus.read() == Focus::Val,
                            block: Block::bordered()
                                    .title(print_ein(ein, palette, true).centered())
                                    .border_style(border_style(palette, *focus.read() == Focus::Val))
                        ) {
                            for (index, fill) in fills.into_iter().enumerate() {
                                View(key: index, height: Constraint::Length(1)) {
                                    Text(text: print_fill(fill, palette))
                                }
                            }
                        }
                    } else {
                        Loading()
                    }
                }
                View(width: match *layout.read() {
                    Layout::OneColumn => Constraint::Percentage(0),
                    Layout::TwoColumns => Constraint::Percentage(70),
                    Layout::ThreeColumns => Constraint::Percentage(20),
                }) {
                    if let (Some(eins), Some(attr)) = (eins.read().clone(), active_attr.read().clone()) {
                        EinSelect(
                            items: eins,
                            selected: *active_ein.read(),
                            top_title: Some(print_attr_title(attr, palette, *focus.read() == Focus::Ein)),
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
