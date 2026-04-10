use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::state::item::{FoodKind, ItemKind, MedKind, ToyKind};

/// All items available in the shop, in display order
pub const SHOP_ITEMS: &[ItemKind] = &[
    ItemKind::Food(FoodKind::OranBerry),
    ItemKind::Food(FoodKind::SitrusBerry),
    ItemKind::Food(FoodKind::PokePuff),
    ItemKind::Toy(ToyKind::Ball),
    ItemKind::Toy(ToyKind::Ribbon),
    ItemKind::Toy(ToyKind::Mirror),
    ItemKind::Medicine(MedKind::Potion),
    ItemKind::Medicine(MedKind::FullRestore),
    ItemKind::Medicine(MedKind::Revive),
];

pub struct ShopState {
    pub selected: usize,
    pub confirm: bool, // waiting for y/n confirmation
}

impl ShopState {
    pub fn new() -> Self {
        Self { selected: 0, confirm: false }
    }

    pub fn up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
        self.confirm = false;
    }

    pub fn down(&mut self) {
        if self.selected + 1 < SHOP_ITEMS.len() {
            self.selected += 1;
        }
        self.confirm = false;
    }

    pub fn selected_item(&self) -> &ItemKind {
        &SHOP_ITEMS[self.selected]
    }
}

pub fn render(f: &mut Frame, area: Rect, shop: &ShopState, wallet: u32) {
    // Centered overlay: 60% width, 70% height
    let width = (area.width as f32 * 0.6) as u16;
    let height = (area.height as f32 * 0.7) as u16;
    let x = area.x + (area.width - width) / 2;
    let y = area.y + (area.height - height) / 2;
    let overlay = Rect { x, y, width, height };

    f.render_widget(Clear, overlay);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Shop  💰 {} coins ", wallet))
        .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    let inner = block.inner(overlay);
    f.render_widget(block, overlay);

    // Split inner: item list | item detail
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(inner);

    // Item list
    let mut list_state = ListState::default();
    list_state.select(Some(shop.selected));

    let items: Vec<ListItem> = SHOP_ITEMS
        .iter()
        .map(|item| {
            let can_afford = wallet >= item.shop_price();
            let color = if can_afford { Color::White } else { Color::DarkGray };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:<16}", item.display_name()),
                    Style::default().fg(color),
                ),
                Span::styled(
                    format!("{:>5}c", item.shop_price()),
                    Style::default().fg(Color::Yellow),
                ),
            ]))
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, cols[0], &mut list_state);

    // Item detail panel
    let item = shop.selected_item();
    let delta = item.stat_delta();
    let can_afford = wallet >= item.shop_price();

    let mut detail_lines = vec![
        Line::from(Span::styled(
            item.display_name(),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled("Effects:", Style::default().fg(Color::Gray))),
    ];

    if delta.hunger != 0.0 {
        detail_lines.push(stat_line("Hunger", delta.hunger));
    }
    if delta.happiness != 0.0 {
        detail_lines.push(stat_line("Happiness", delta.happiness));
    }
    if delta.energy != 0.0 {
        detail_lines.push(stat_line("Energy", delta.energy));
    }

    detail_lines.push(Line::from(""));

    if shop.confirm {
        detail_lines.push(Line::from(Span::styled(
            "Buy? [y] Yes  [n] No",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::RAPID_BLINK),
        )));
    } else if can_afford {
        detail_lines.push(Line::from(Span::styled(
            "[Enter] Buy",
            Style::default().fg(Color::Green),
        )));
    } else {
        detail_lines.push(Line::from(Span::styled(
            "Not enough coins",
            Style::default().fg(Color::Red),
        )));
    }

    detail_lines.push(Line::from(""));
    detail_lines.push(Line::from(Span::styled(
        "[↑↓] Navigate  [Esc] Close",
        Style::default().fg(Color::DarkGray),
    )));

    f.render_widget(Paragraph::new(detail_lines).alignment(Alignment::Left), cols[1]);
}

fn stat_line(name: &str, delta: f32) -> Line<'static> {
    let (sign, color) = if delta > 0.0 {
        ("+", Color::Green)
    } else {
        ("-", Color::Red)
    };
    Line::from(vec![
        Span::styled(format!("  {:<10} ", name), Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}{:.0}", sign, delta.abs()),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ])
}
