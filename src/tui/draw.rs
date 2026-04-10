use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::app::App;
use super::widgets::{help_bar, party_panel, play_area, shop_overlay, status_bar};

pub fn draw(f: &mut Frame, app: &mut App) {
    app.tick_notifications();

    let area = f.area();

    // Root: play area (left) | sidebar (right)
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(40),
            Constraint::Length(30),
        ])
        .split(area);

    // Sidebar: party panel | status bars | help
    let sidebar = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),
            Constraint::Min(8),
            Constraint::Length(1),
        ])
        .split(h_chunks[1]);

    let play_rect = h_chunks[0];
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

    party_panel::render(f, sidebar[0], &app.party);
    status_bar::render(f, sidebar[1], app.party.active_pet(), app.party.wallet, &app.party.inventory);
    help_bar::render(f, sidebar[2], app.shop.is_some());

    // Notification overlay
    if let Some(notif) = &app.notification {
        render_notification(f, play_rect, &notif.message);
    }

    // Shop overlay (rendered last = on top)
    if let Some(shop) = &app.shop {
        shop_overlay::render(f, area, shop, app.party.wallet);
    }
}

fn render_notification(f: &mut Frame, play_rect: Rect, msg: &str) {
    let width = (msg.len() as u16 + 4).min(play_rect.width.saturating_sub(4));
    let height = 3u16;
    let x = play_rect.x + (play_rect.width.saturating_sub(width)) / 2;
    let y = play_rect.y + play_rect.height.saturating_sub(height + 2);
    let area = Rect { x, y, width, height };

    f.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));
    let inner = block.inner(area);
    f.render_widget(block, area);
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            msg,
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ))),
        inner,
    );
}
