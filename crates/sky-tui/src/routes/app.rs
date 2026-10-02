use crate::routes::browser::Browser;
use ratatui_kit::crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui_kit::prelude::*;
use sky_db::Db;
use sky_db::DbReader;
use sky_db::trie_storage::MemView;
use sky_db::{Transact, attr, datom, ent};

pub static DB_VIEW: Atom<Option<DbReader<MemView>>> = Atom::new(|| None);

#[component]
pub fn App(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut exit = hooks.use_exit();
    let mut db_view = hooks.use_atom(&DB_VIEW);

    hooks.use_future(async move {
        let attr_count = attr("counter/count");
        let mut db = Db::new(MemView::new(), [attr_count.clone()])
            .await
            .expect("db");
        db.transact([datom::add(ent("a"), attr_count, 33)])
            .await
            .expect("transact");
        let snap = db.to_reader();
        db_view.set(Some(snap));
    });

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
