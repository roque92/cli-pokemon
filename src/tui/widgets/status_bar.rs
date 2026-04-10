use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};
use ratatui::layout::Rect;

use crate::state::{item::Inventory, pet::{Pet, PetStatus}};

pub fn render(f: &mut Frame, area: Rect, pet: Option<&Pet>, wallet: u32, inventory: &Inventory) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Status ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let Some(pet) = pet else {
        f.render_widget(
            Paragraph::new("No active Pokemon.").style(Style::default().fg(Color::DarkGray)),
            inner,
        );
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // name + level
            Constraint::Length(1), // hunger
            Constraint::Length(1), // happiness
            Constraint::Length(1), // energy
            Constraint::Length(1), // wallet + item count
        ])
        .split(inner);

    // Name line
    let status_icon = match &pet.status {
        PetStatus::Egg { hatch_steps } => format!(" 🥚{}", hatch_steps),
        PetStatus::Alive => String::new(),
        PetStatus::Fainted => " 💀".to_string(),
    };
    let shiny = if pet.shiny { " ✦" } else { "" };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!("{}{}", pet.display_name(), shiny),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(" Lv.{}{}", pet.level, status_icon)),
        ])),
        chunks[0],
    );

    // Stat bars
    if chunks.len() > 3 {
        render_bar(f, chunks[1], "Hunger   ", pet.stats.hunger, hunger_color(pet.stats.hunger));
        render_bar(f, chunks[2], "Happiness", pet.stats.happiness, Color::Magenta);
        render_bar(f, chunks[3], "Energy   ", pet.stats.energy, Color::Cyan);
    }

    // Wallet + item count
    if chunks.len() > 4 {
        let item_count: u32 = inventory.stacks.iter().map(|s| s.qty).sum();
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("💰 ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{}", wallet),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled("🎒 ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{} items", item_count),
                    Style::default().fg(Color::Cyan),
                ),
            ])),
            chunks[4],
        );
    }
}

fn render_bar(f: &mut Frame, area: Rect, label: &str, value: f32, color: Color) {
    let gauge = Gauge::default()
        .label(format!("{} {:>5.1}%", label, value))
        .ratio((value / 100.0).clamp(0.0, 1.0) as f64)
        .gauge_style(Style::default().fg(color).bg(Color::DarkGray));
    f.render_widget(gauge, area);
}

fn hunger_color(h: f32) -> Color {
    if h < 20.0 { Color::Red } else if h < 50.0 { Color::Yellow } else { Color::Green }
}
