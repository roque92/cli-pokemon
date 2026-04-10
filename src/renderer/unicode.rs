use std::collections::HashMap;

use image::{Rgba, RgbaImage};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::animation::state_machine::AnimState;
use crate::sprite::gif::DecodedFrame;

const TARGET_COLS: u32 = 24;
const TARGET_ROWS: u32 = 12; // ×2 pixel rows via half-blocks

pub struct UnicodeRenderer {
    /// Per-state pre-rendered frames: state → Vec<frame_rows>
    frames: HashMap<AnimState, Vec<Vec<Line<'static>>>>,
}

impl UnicodeRenderer {
    /// Pre-render frames for all states at once from the SpriteSheet.
    pub fn from_sheet(sheet: &crate::sprite::sheet::SpriteSheet) -> Self {
        let states = [
            AnimState::Idle,
            AnimState::WalkLeft,
            AnimState::WalkRight,
            AnimState::Sleep,
            AnimState::Eat,
            AnimState::HappyBounce,
            AnimState::Hatch,
        ];

        let mut frames = HashMap::new();
        for state in states {
            let raw = sheet.frames_for(state);
            if !raw.is_empty() {
                frames.insert(state, raw.iter().map(|f| render_frame(&f.pixels)).collect());
            }
        }

        Self { frames }
    }

    /// Pre-render from a flat frame list (Idle only — for backwards compat).
    pub fn from_frames(raw_frames: &[DecodedFrame]) -> Self {
        let rendered: Vec<Vec<Line<'static>>> =
            raw_frames.iter().map(|f| render_frame(&f.pixels)).collect();

        let mut frames = HashMap::new();
        for state in [AnimState::Idle, AnimState::WalkLeft, AnimState::WalkRight,
                       AnimState::Sleep, AnimState::Eat, AnimState::HappyBounce, AnimState::Hatch] {
            frames.insert(state, rendered.clone());
        }

        Self { frames }
    }

    /// Get the rendered lines for the current animation state + frame index.
    pub fn get_lines(&self, state: AnimState, frame_index: usize) -> &[Line<'static>] {
        let bank = self
            .frames
            .get(&state)
            .or_else(|| self.frames.get(&AnimState::Idle));

        match bank {
            Some(bank) if !bank.is_empty() => {
                &bank[frame_index.min(bank.len() - 1)]
            }
            _ => &[],
        }
    }
}

fn render_frame(img: &RgbaImage) -> Vec<Line<'static>> {
    let scaled = image::imageops::resize(
        img,
        TARGET_COLS,
        TARGET_ROWS * 2,
        image::imageops::FilterType::Nearest,
    );

    let width = scaled.width();
    let height = scaled.height();
    let mut lines = Vec::new();

    let mut y = 0u32;
    while y + 1 < height {
        let mut spans: Vec<Span<'static>> = Vec::new();
        for x in 0..width {
            let upper = scaled.get_pixel(x, y);
            let lower = scaled.get_pixel(x, y + 1);
            let fg = if upper[3] < 128 { Color::Reset } else { rgba_to_ansi(upper) };
            let bg = if lower[3] < 128 { Color::Reset } else { rgba_to_ansi(lower) };
            spans.push(Span::styled("▀", Style::default().fg(fg).bg(bg)));
        }
        lines.push(Line::from(spans));
        y += 2;
    }

    lines
}

fn rgba_to_ansi(px: &Rgba<u8>) -> Color {
    fn step(c: u8) -> u8 {
        if c < 48 { 0 } else if c < 115 { 1 } else if c < 155 { 2 }
        else if c < 195 { 3 } else if c < 235 { 4 } else { 5 }
    }
    let index = 16 + 36 * step(px[0]) + 6 * step(px[1]) + step(px[2]);
    Color::Indexed(index)
}
