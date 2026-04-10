/// Compact layout — minimal pane for tmux/zellij splits.
/// No sidebar. Pet roams the full area with a single status line at the bottom.
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Gauge, Paragraph},
    Frame,
};


use super::app::App;
use super::widgets::play_area;

pub fn draw(f: &mut Frame, app: &mut App) {
    app.tick_notifications();

    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // play area
            Constraint::Length(1), // status line
        ])
        .split(area);

    let play_rect = chunks[0];
    app.set_play_bounds(Rect {
        x: play_rect.x + 1,
        y: play_rect.y + 1,
        width: play_rect.width.saturating_sub(2),
        height: play_rect.height.saturating_sub(2),
    });

    let pet_pos = app.walk.as_ref().map(|w| w.pos).unwrap_or_default();
    let anim_state = app.ticker.current_state;
    let frame_idx = app.ticker.frame_index;
    let active_pet = app.party.active_pet();

    play_area::render(
        f,
        play_rect,
        app.renderer_kind,
        app.unicode_renderer.as_ref(),
        anim_state,
        frame_idx,
        pet_pos,
        active_pet,
        app.loading_sprite,
    );

    // Single status line: name | hunger bar | coins | keyhints
    render_status_line(f, chunks[1], app);

    // Notification (compact — just text, no box)
    if let Some(notif) = &app.notification {
        let area = Rect {
            x: play_rect.x + 1,
            y: play_rect.y + play_rect.height.saturating_sub(2),
            width: play_rect.width.saturating_sub(2),
            height: 1,
        };
        f.render_widget(
            Paragraph::new(Span::styled(
                &notif.message,
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )),
            area,
        );
    }
}

fn render_status_line(f: &mut Frame, area: Rect, app: &App) {
    let pet = match app.party.active_pet() {
        Some(p) => p,
        None => {
            f.render_widget(
                Paragraph::new(Span::styled(
                    "No Pokemon — run `petcli catch`",
                    Style::default().fg(Color::DarkGray),
                )),
                area,
            );
            return;
        }
    };

    // Split: name | hunger gauge | coins | hints
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(14),
            Constraint::Min(10),
            Constraint::Length(8),
            Constraint::Length(18),
        ])
        .split(area);

    let shiny = if pet.shiny { "✦ " } else { "" };
    f.render_widget(
        Paragraph::new(Span::styled(
            format!("{}{}", shiny, pet.display_name()),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        cols[0],
    );

    let hunger_color = if pet.stats.hunger < 20.0 { Color::Red }
        else if pet.stats.hunger < 50.0 { Color::Yellow }
        else { Color::Green };

    f.render_widget(
        Gauge::default()
            .label(format!("H{:.0}%", pet.stats.hunger))
            .ratio((pet.stats.hunger / 100.0).clamp(0.0, 1.0) as f64)
            .gauge_style(Style::default().fg(hunger_color).bg(Color::DarkGray)),
        cols[1],
    );

    f.render_widget(
        Paragraph::new(Span::styled(
            format!("💰{}", app.party.wallet),
            Style::default().fg(Color::Yellow),
        )),
        cols[2],
    );

    f.render_widget(
        Paragraph::new(Span::styled(
            "[f]Feed [p]Pet [q]Quit",
            Style::default().fg(Color::DarkGray),
        )),
        cols[3],
    );
}
