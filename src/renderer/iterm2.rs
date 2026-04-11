//! iTerm2 inline image protocol renderer.
//!
//! ESC ] 1337 ; File = inline=1;width=<W>px;height=<H>px : <base64-png> BEL
//!
//! Supported by: iTerm2, Warp, Tabby, and other macOS/cross-platform terminals
//! that implement the iTerm2 image protocol.

use std::io::Write;

use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use crossterm::{cursor::MoveTo, queue, style::Print};
use image::{imageops, ImageEncoder, RgbaImage};

use crate::sprite::{gif::DecodedFrame, sheet::SpriteSheet};

/// Native sprite pixel size uploaded to the terminal.
const SPRITE_PX: u32 = 48;

/// Approximate terminal cell footprint for a 48×48px image.
/// At typical macOS terminal cell sizes (~8px wide, ~16px tall):
///   cols = 48 / 8 = 6   rows = 48 / 16 = 3
pub const SPRITE_COLS: u16 = 6;
pub const SPRITE_ROWS: u16 = 3;

// ── Pre-rendered frame cache ──────────────────────────────────────────────────

/// All animation variants for one pet, pre-encoded as base64 PNG strings.
/// Encoding happens once at load time; drawing just emits the string.
pub struct Iterm2Frames {
    idle:      Vec<String>,
    idle_flip: Vec<String>,
    walk:      Vec<String>,
    walk_flip: Vec<String>,
}

impl Iterm2Frames {
    pub fn prerender(sheet: &SpriteSheet) -> Self {
        let encode = |frames: &[DecodedFrame], flip: bool| -> Vec<String> {
            frames.iter().map(|f| encode_frame(&f.pixels, flip)).collect()
        };
        Self {
            idle:      encode(sheet.idle_frames(), false),
            idle_flip: encode(sheet.idle_frames(), true),
            walk:      encode(sheet.walk_frames(), false),
            walk_flip: encode(sheet.walk_frames(), true),
        }
    }

    pub fn get(&self, is_walk: bool, flip: bool, idx: usize) -> &str {
        let bank = match (is_walk, flip) {
            (false, false) => &self.idle,
            (false, true)  => &self.idle_flip,
            (true,  false) => &self.walk,
            (true,  true)  => &self.walk_flip,
        };
        // Fall back to idle if walk is empty
        let bank = if bank.is_empty() { &self.idle } else { bank };
        if bank.is_empty() { return ""; }
        &bank[idx % bank.len()]
    }
}

// ── Draw / clear ──────────────────────────────────────────────────────────────

/// Emit an iTerm2 inline image at terminal cell position (col, row).
pub fn draw_frame(out: &mut impl Write, encoded: &str, col: u16, row: u16) -> Result<()> {
    if encoded.is_empty() {
        return Ok(());
    }
    // Move cursor to target position
    queue!(out, MoveTo(col, row))?;
    // iTerm2 inline image escape
    write!(
        out,
        "\x1b]1337;File=inline=1;width={SPRITE_PX}px;height={SPRITE_PX}px;preserveAspectRatio=1:{encoded}\x07"
    )?;
    Ok(())
}

/// Erase the sprite cell area (col, row) by overwriting with spaces.
pub fn clear_frame(out: &mut impl Write, col: u16, row: u16) -> Result<()> {
    let blank = " ".repeat(SPRITE_COLS as usize + 1);
    for r in 0..SPRITE_ROWS {
        queue!(out, MoveTo(col, row + r), Print(&blank))?;
    }
    Ok(())
}

// ── Internal ──────────────────────────────────────────────────────────────────

fn encode_frame(img: &RgbaImage, flip: bool) -> String {
    let scaled = if img.width() == SPRITE_PX && img.height() == SPRITE_PX {
        img.clone()
    } else {
        imageops::resize(img, SPRITE_PX, SPRITE_PX, imageops::FilterType::Nearest)
    };
    let prepared = if flip {
        imageops::flip_horizontal(&scaled)
    } else {
        scaled
    };

    let mut buf = Vec::with_capacity(SPRITE_PX as usize * SPRITE_PX as usize * 4);
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    let _ = encoder.write_image(
        prepared.as_raw(),
        prepared.width(),
        prepared.height(),
        image::ExtendedColorType::Rgba8,
    );
    B64.encode(&buf)
}
