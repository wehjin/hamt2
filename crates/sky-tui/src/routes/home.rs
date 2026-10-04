use crate::components::loading::Loading;
use crate::routes::app::DB_VIEW;
use crate::routes::attribute_binds_table::AttributeBindsTable;
use ratatui_kit::prelude::*;
use ratatui_kit::ratatui::prelude::{Constraint, Direction, Line, Style, Stylize};
use ratatui_kit::ratatui::style::Styled;
use ratatui_kit::ratatui::widgets::Padding;
use sky_db::Attribute;
use sky_db::cardinality::Cardinality;

#[component]
pub fn Home(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let pod = hooks.use_atom(&DB_VIEW);
    let mut attributes = hooks.use_state(|| None::<Vec<Attribute>>);
    let mut selected_index = hooks.use_state(|| None::<usize>);
    let mut detail = hooks.use_state(|| None::<AttrDetails>);
    let mut attribute = hooks.use_state(|| None::<Attribute>);

    let attrs_pod = pod.read().clone();
    let attrs_deps = attrs_pod.clone();
    hooks.use_async_effect(
        async move {
            if let Some(pod) = attrs_pod {
                let next_attributes = pod.list_attributes().await;
                attributes.set(Some(next_attributes.clone()));
                if next_attributes.len() > 0 {
                    selected_index.set(Some(0));
                } else {
                    selected_index.set(None);
                }
            } else {
                attributes.set(None);
                selected_index.set(None);
            }
        },
        attrs_deps,
    );

    {
        let fx_attributes = attributes.read().clone();
        let fx_selected_index = selected_index.read().clone();
        let fx_deps = (fx_attributes.clone(), fx_selected_index.clone());
        hooks.use_effect(
            || match (fx_attributes, fx_selected_index) {
                (Some(attributes), Some(index)) if index < attributes.len() => {
                    let selected = attributes[index].clone();
                    detail.set(Some(AttrDetails::from(&selected)));
                    attribute.set(Some(selected));
                }
                _ => {
                    detail.set(None);
                    attributes.set(None);
                }
            },
            fx_deps,
        );
    }

    let palette = hooks.use_palette();
    element!(
        Border(
            border_style: Style::new().fg(palette.surface),
            top_title: Line::from( " Pod ").centered(),
        ) {
            if let Some(attributes) = attributes.read().clone() {
                View(
                    flex_direction: Direction::Horizontal,
                ) {
                    Select<String>(
                        width: Constraint::Length(20),
                        items: attributes.iter().map(|it|it.ident().to_string()).collect::<Vec<_>>(),
                        default_index: selected_index.get(),
                        border_style: Style::new().fg(palette.surface),
                        top_title: Line::from(" Attributes ").centered(),
                        empty_message: "None",
                        on_select: move |item: String| {
                            let index = attributes.iter().position(|attr| {
                                attr.ident() == item
                            });
                            selected_index.set(index);
                        }
                    )
                    View(
                        flex_direction: Direction::Vertical,
                    ) {
                        if let Some(detail) = detail.read().clone() {
                            Border(
                                padding: Padding::new(1, 1, 0, 0),
                                border_style: Style::new().fg(palette.surface),
                                top_title: Line::from(format!(" {} ", detail.ident.clone()).fg(palette.selection)).centered(),
                            ) {
                                Detail(label: "cardinality".to_string(), value: detail.cardinality.clone())
                            }
                        }
                        if let Some(attribute) = attribute.read().clone() {
                            ContextProvider(value: Context::owned(attribute)) {
                               AttributeBindsTable()
                            }
                        }
                    }
                }
            } else {
                Loading()
            }
        }
    )
}

#[derive(Default, Clone)]
struct AttrDetails {
    pub ident: String,
    pub cardinality: String,
}
impl From<&Attribute> for AttrDetails {
    fn from(attr: &Attribute) -> Self {
        let ident = attr.ident().to_string();
        let cardinality = card_str(&attr.cardinality()).to_string();
        Self { ident, cardinality }
    }
}

fn card_str(card: &Cardinality) -> &'static str {
    match card {
        Cardinality::One => "one",
        Cardinality::Many => "many",
    }
}

#[derive(Default, Props)]
struct DetailProps {
    pub label: String,
    pub value: String,
}

#[component]
fn Detail(props: &DetailProps, hooks: &Hooks) -> impl Into<AnyElement<'static>> {
    let palette = hooks.use_palette();
    let line = Line::from(vec![
        format!("{}:", props.label).into(),
        " ".into(),
        props
            .value
            .clone()
            .set_style(Style::new().fg(palette.accent)),
    ]);
    element!(
        View (height: Constraint::Length(1)) {
            Text (text: line)
        }
    )
}
