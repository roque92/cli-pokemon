use std::io::Write;

use anyhow::Result;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use image::{imageops, Rgba, RgbaImage};

/// Sprite display dimensions in terminal cells.
/// 48px source scaled to 24 cols × 24 pixel rows → 12 half-block rows.
pub const SPRITE_COLS: u16 = 24;
pub const SPRITE_ROWS: u16 = 12;

/// A rendered frame: rows of (fg, bg, char) cells.
pub type RenderedRow = Vec<(Color, Color, char)>;
pub type RenderedFrame = Vec<RenderedRow>;

/// Pre-render a decoded GIF frame into half-block terminal cells.
/// Set `flip = true` to mirror horizontally (for left-facing walk).
pub fn render_frame(img: &RgbaImage, flip: bool) -> RenderedFrame {
    let w = SPRITE_COLS as u32;
    let h = (SPRITE_ROWS * 2) as u32; // 2 pixel rows per half-block row

    let scaled = if flip {
        let flipped = imageops::flip_horizontal(img);
        imageops::resize(&flipped, w, h, imageops::FilterType::Nearest)
    } else {
        imageops::resize(img, w, h, imageops::FilterType::Nearest)
    };

    let height = scaled.height();
    let width = scaled.width();
    let mut rows = Vec::with_capacity(SPRITE_ROWS as usize);

    let mut y = 0u32;
    while y + 1 < height {
        let mut row = Vec::with_capacity(width as usize);
        for x in 0..width {
            let upper = scaled.get_pixel(x, y);
            let lower = scaled.get_pixel(x, y + 1);

            let u_opaque = upper[3] >= 128;
            let l_opaque = lower[3] >= 128;

            let (ch, fg, bg) = match (u_opaque, l_opaque) {
                (true, true)   => ('▀', ansi_color(upper), ansi_color(lower)),
                (true, false)  => ('▀', ansi_color(upper), Color::Reset),
                (false, true)  => ('▄', ansi_color(lower), Color::Reset),
                (false, false) => (' ', Color::Reset, Color::Reset),
            };
            row.push((fg, bg, ch));
        }
        rows.push(row);
        y += 2;
    }

    rows
}

/// Write a pre-rendered frame to the terminal at position (col, row).
pub fn draw_frame(out: &mut impl Write, frame: &RenderedFrame, col: u16, row: u16) -> Result<()> {
    for (r, line) in frame.iter().enumerate() {
        queue!(out, MoveTo(col, row + r as u16))?;
        for (fg, bg, ch) in line {
            queue!(
                out,
                SetForegroundColor(*fg),
                SetBackgroundColor(*bg),
                Print(ch)
            )?;
        }
        queue!(out, ResetColor)?;
    }
    Ok(())
}

/// Erase the sprite area at (col, row) by overwriting with spaces.
pub fn clear_frame(out: &mut impl Write, col: u16, row: u16) -> Result<()> {
    let blank: String = " ".repeat(SPRITE_COLS as usize + 1);
    for r in 0..SPRITE_ROWS {
        queue!(out, MoveTo(col, row + r), Print(&blank))?;
    }
    Ok(())
}

fn ansi_color(px: &Rgba<u8>) -> Color {
    fn step(c: u8) -> u8 {
        if c < 48 { 0 }
        else if c < 115 { 1 }
        else if c < 155 { 2 }
        else if c < 195 { 3 }
        else if c < 235 { 4 }
        else { 5 }
    }
    Color::AnsiValue(16 + 36 * step(px[0]) + 6 * step(px[1]) + step(px[2]))
}
