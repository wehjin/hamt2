use crate::components::loading::Loading;
use ratatui_kit::prelude::*;
use ratatui_kit::ratatui::layout::Constraint;
use ratatui_kit::ratatui::prelude::{Line, Style, Stylize};
use ratatui_kit::ratatui::widgets::Padding;
use sky_db::cardinality::Cardinality;
use sky_db::{Attribute, Val};
use std::sync::Arc;

#[component]
pub fn AttributeBindsTable(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let attribute = hooks.use_context::<Attribute>();
    let mut infos = hooks.use_state(|| None::<Vec<BindInfo>>);
    let palette = hooks.use_palette();

    {
        let fx_attribute = attribute.clone();
        let fx_deps = fx_attribute.clone();
        hooks.use_async_effect(
            async move {
                let mut binds = fx_attribute.list_binds().await;
                binds.sort();
                let mut new_infos = vec![];
                for bind in binds {
                    let info = BindInfo {
                        ein_string: format!("◆ {}", bind.0.to_i32()),
                        val_string: print_val(&fx_attribute, &bind.1),
                    };
                    new_infos.push(info);
                }
                infos.set(Some(new_infos));
            },
            fx_deps,
        );
    }

    element! {
        Border(
            padding: Padding::new(1, 1, 0, 0),
            border_style: Style::new().fg(palette.surface),
            top_title: Line::from(format!(" [{}] ", attribute.ident()).bold()).centered(),
        ) {
            if let Some(infos) = infos.read().clone() {
                BindsTable(infos:infos.clone())
            } else {
                Loading()
            }
        }
    }
}

#[derive(Default, Clone, PartialEq)]
struct BindInfo {
    pub ein_string: String,
    pub val_string: String,
}

fn print_val(attribute: &Attribute, val: &Val) -> String {
    if attribute.ident() == "db/cardinality" {
        let card = Cardinality::from(val.clone());
        match card {
            Cardinality::One => "cardinality/one",
            Cardinality::Many => "cardinality/many",
        }
        .to_string()
    } else {
        match val {
            Val::U32(val) => format!("#{}", val),
            Val::String(val) => format!("\"{}\"", val),
        }
    }
}

#[derive(Default, Props)]
struct BindsTableProps {
    pub infos: Vec<BindInfo>,
}

#[component]
fn BindsTable(props: &BindsTableProps) -> impl Into<AnyElement<'static>> {
    let columns = vec![
        TableColumn::new("Entity", Constraint::Percentage(50)),
        TableColumn::new("Value", Constraint::Percentage(50)),
    ];
    let render_row: RenderTableRow<BindInfo> = Arc::new(|info, _selected| {
        vec![
            TableCell::new(info.ein_string.clone()),
            TableCell::new(info.val_string.clone()),
        ]
    });
    element!(
        Table<BindInfo>(
            columns: columns,
            rows: props.infos.clone(),
            render_row: Some(render_row),
        )
    )
}
