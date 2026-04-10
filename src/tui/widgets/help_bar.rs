use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, shop_open: bool) {
    let keys: &[(&str, &str)] = if shop_open {
        &[("↑↓", "Navigate"), ("Enter", "Buy"), ("n", "Cancel"), ("Esc", "Close")]
    } else {
        &[("f", "Feed"), ("p", "Pet"), ("u", "Use item"), ("s", "Shop"), ("Tab", "Cycle"), ("q", "Quit")]
    };

    let mut spans: Vec<Span> = Vec::new();
    for (i, (key, label)) in keys.iter().enumerate() {
        if i > 0 { spans.push(Span::raw("  ")); }
        spans.push(Span::styled(
            format!("[{}]", key),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(format!(" {}", label), Style::default().fg(Color::Gray)));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}
