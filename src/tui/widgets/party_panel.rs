use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::state::{party::Party, pet::PetStatus};

pub fn render(f: &mut Frame, area: Rect, party: &Party) {
    let items: Vec<ListItem> = party
        .slots
        .iter()
        .enumerate()
        .filter_map(|(i, slot)| {
            let pet = slot.as_ref()?;
            let is_active = i == party.active;

            let status_icon = match &pet.status {
                PetStatus::Egg { hatch_steps } => format!("🥚{}", hatch_steps),
                PetStatus::Alive => String::new(),
                PetStatus::Fainted => "💀".to_string(),
            };

            let shiny = if pet.shiny { "✦ " } else { "" };
            let marker = if is_active { "▶ " } else { "  " };
            let name = pet.display_name();

            let style = if is_active {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            let text = format!(
                "{}{}{}{}  Lv.{}",
                marker, shiny, name, status_icon, pet.level
            );

            Some(ListItem::new(Line::from(Span::styled(text, style))))
        })
        .collect();

    let count = party.count();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Party ({}/6) ", count));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}
