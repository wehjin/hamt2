use ratatui_kit::Palette;
use ratatui_kit::ratatui::prelude::{Line, Span, Style};
use sky_types::db::{Ein, Fill, Val};

pub fn print_ein<'a>(ein: Ein, palette: Palette, margins: bool) -> Line<'a> {
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

pub fn print_fill<'a>(fill: Fill, palette: Palette) -> Line<'a> {
    let Fill(attr, val) = fill;
    let attr_string = attr.to_string();
    let val_string = match val {
        Val::U32(val) => format!("{}", val),
        Val::String(val) => format!("\"{}\"", val),
    };
    let attr_style = Style::new().fg(palette.accent);
    let normal_style = Style::new();
    let val_style = Style::new().fg(palette.selection);

    let spans = vec![
        Span::styled(attr_string, attr_style),
        Span::styled(" = ", normal_style),
        Span::styled(val_string, val_style),
    ];
    Line::from(spans)
}
