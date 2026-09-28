use ratatui_kit::Palette;
use ratatui_kit::ratatui::style::Style;

pub fn highlight_style(palette: Palette, focused: bool) -> Style {
    if focused {
        Style::new().fg(palette.on_accent).bg(palette.selection)
    } else {
        Style::new().fg(palette.fg_dim).bg(palette.selection)
    }
}

pub fn border_style(palette: Palette, focused: bool) -> Style {
    if focused {
        Style::new().fg(palette.border_active)
    } else {
        Style::new().fg(palette.border)
    }
}
