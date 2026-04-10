use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    animation::{state_machine::AnimState, walk::TermPos},
    renderer::{unicode::UnicodeRenderer, RendererKind},
    state::pet::{Pet, PetStatus},
};

pub fn render(
    f: &mut Frame,
    area: Rect,
    renderer_kind: RendererKind,
    unicode_renderer: Option<&UnicodeRenderer>,
    anim_state: AnimState,
    frame_index: usize,
    pet_pos: TermPos,
    active_pet: Option<&Pet>,
    loading_sprite: bool,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" petcli ");
    f.render_widget(block, area);

    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    match active_pet {
        None => {
            let msg = Paragraph::new("No Pokemon in party.\nRun `petcli catch` outside the TUI to get one!")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            let centered = center_rect(inner, 50, 3);
            f.render_widget(msg, centered);
        }

        Some(pet) if matches!(pet.status, PetStatus::Egg { .. }) => {
            let steps = if let PetStatus::Egg { hatch_steps } = &pet.status {
                *hatch_steps
            } else { 0 };

            let egg_art = vec![
                Line::from(Span::styled("    ,-.    ", Style::default().fg(Color::White))),
                Line::from(Span::styled("   /   \\   ", Style::default().fg(Color::White))),
                Line::from(Span::styled("  |     |  ", Style::default().fg(Color::White))),
                Line::from(Span::styled("   \\   /   ", Style::default().fg(Color::White))),
                Line::from(Span::styled("    `-'    ", Style::default().fg(Color::White))),
            ];

            let cx = inner.x + inner.width / 2 - 5;
            let cy = inner.y + inner.height / 2 - 3;
            for (i, line) in egg_art.iter().enumerate() {
                let row = cy + i as u16;
                if row < inner.y + inner.height {
                    f.render_widget(
                        Paragraph::new(line.clone()),
                        Rect { x: cx, y: row, width: 11, height: 1 },
                    );
                }
            }

            let hint = Paragraph::new(Line::from(vec![
                Span::styled(
                    format!("🥚 {} · {} steps to hatch", pet.display_name(), steps),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
            ]))
            .alignment(Alignment::Center);
            let hint_area = Rect {
                x: inner.x,
                y: inner.y + inner.height.saturating_sub(3),
                width: inner.width,
                height: 1,
            };
            f.render_widget(hint, hint_area);

            let sub = Paragraph::new("Press [f] or [p] to interact and hatch!")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            let sub_area = Rect {
                x: inner.x,
                y: inner.y + inner.height.saturating_sub(2),
                width: inner.width,
                height: 1,
            };
            f.render_widget(sub, sub_area);
        }

        Some(pet) if matches!(pet.status, PetStatus::Fainted) => {
            let msg = Paragraph::new(format!(
                "💀 {} has fainted!\nUse a Revive to restore them.",
                pet.display_name()
            ))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
            let centered = center_rect(inner, 40, 3);
            f.render_widget(msg, centered);
        }

        Some(_) => {
            // Alive — render sprite
            match renderer_kind {
                RendererKind::Kitty => {
                    if loading_sprite {
                        let loading = Paragraph::new(Span::styled(
                            "Loading sprite...",
                            Style::default().fg(Color::DarkGray),
                        ))
                        .alignment(Alignment::Center);
                        f.render_widget(loading, center_rect(inner, 20, 1));
                    }
                    // APC sequences for the actual sprite are written post-draw in run.rs
                }
                RendererKind::Unicode => {
                    if let Some(ur) = unicode_renderer {
                        let lines = ur.get_lines(anim_state, frame_index);
                        let sprite_h = lines.len() as u16;
                        let sprite_w = 24u16;

                        let start_row = pet_pos
                            .row()
                            .saturating_sub(inner.y)
                            .min(inner.height.saturating_sub(sprite_h));
                        let start_col = pet_pos
                            .col()
                            .saturating_sub(inner.x)
                            .min(inner.width.saturating_sub(sprite_w));

                        for (i, line) in lines.iter().enumerate() {
                            let row = inner.y + start_row + i as u16;
                            if row >= inner.y + inner.height { break; }
                            f.render_widget(
                                Paragraph::new(line.clone()),
                                Rect {
                                    x: inner.x + start_col,
                                    y: row,
                                    width: inner.width.saturating_sub(start_col),
                                    height: 1,
                                },
                            );
                        }
                    } else {
                        // Sprite not loaded yet
                        let col = pet_pos.col().min(inner.width.saturating_sub(3));
                        let row = pet_pos.row().min(inner.height.saturating_sub(1));
                        f.render_widget(
                            Paragraph::new(Span::styled(
                                "⬤",
                                Style::default().fg(Color::Red).add_modifier(Modifier::RAPID_BLINK),
                            )),
                            Rect { x: inner.x + col, y: inner.y + row, width: 3, height: 1 },
                        );
                    }
                }
            }
        }
    }
}

fn center_rect(area: Rect, width: u16, height: u16) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect { x, y, width: width.min(area.width), height: height.min(area.height) }
}
