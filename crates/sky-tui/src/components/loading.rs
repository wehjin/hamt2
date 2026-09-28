use ratatui_kit::prelude::*;
use ratatui_kit::ratatui::layout::Constraint;
use ratatui_kit::ratatui::style::Style;
use ratatui_kit::ratatui::text::Line;

#[component]
pub fn Loading(hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let palette = hooks.use_palette();
    element!(
        Center(
            width: Constraint::Length(24),
            height: Constraint::Length(9),
        ) {
            Text(
                text: Line::styled(
                    "loading".to_string(),
                    Style::new().fg(palette.accent),
                )
                .centered(),
            )
        }
    )
}
