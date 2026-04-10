use std::collections::HashSet;
use std::io::Write;

use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use image::RgbaImage;

use crate::animation::walk::TermPos;
use crate::sprite::gif::DecodedFrame;

const CHUNK_SIZE: usize = 4096;
/// Pixel size to scale sprites to before upload
const TARGET_PX: u32 = 96;

pub struct KittyRenderer {
    /// Image IDs whose animation frames are fully uploaded to the terminal
    uploaded: HashSet<u32>,
    /// Last displayed image id + position — used to decide whether to re-place
    last_image_id: Option<u32>,
    last_pos: Option<(u16, u16)>,
}

impl KittyRenderer {
    pub fn new() -> Self {
        Self {
            uploaded: HashSet::new(),
            last_image_id: None,
            last_pos: None,
        }
    }

    /// Upload all GIF frames as a Kitty animation (once per image_id).
    /// Kitty handles the frame timing internally — we just re-place on position change.
    pub fn upload_animation(
        &mut self,
        image_id: u32,
        frames: &[DecodedFrame],
        out: &mut impl Write,
    ) -> Result<()> {
        if self.uploaded.contains(&image_id) {
            return Ok(());
        }

        for (i, frame) in frames.iter().enumerate() {
            let scaled = scale_to_target(&frame.pixels);
            let rgba = scaled.as_raw();
            let encoded = B64.encode(rgba);
            let chunks: Vec<&[u8]> = encoded.as_bytes().chunks(CHUNK_SIZE).collect();
            let total = chunks.len();
            let w = scaled.width();
            let h = scaled.height();
            // Frame gap in ms (delay before this frame is shown after the previous)
            let gap = frame.delay_ms;

            for (ci, chunk) in chunks.iter().enumerate() {
                let more = if ci < total - 1 { 1u8 } else { 0u8 };
                let chunk_str = std::str::from_utf8(chunk).unwrap_or("");

                if i == 0 && ci == 0 {
                    // First chunk of first frame: transmit with metadata
                    write!(
                        out,
                        "\x1b_Ga=T,f=32,i={},s={},v={},q=2,m={};{}\x1b\\",
                        image_id, w, h, more, chunk_str
                    )?;
                } else if i == 0 {
                    // Continuation chunks of first frame
                    write!(out, "\x1b_Gm={};{}\x1b\\", more, chunk_str)?;
                } else if ci == 0 {
                    // First chunk of subsequent frames: add frame with gap
                    write!(
                        out,
                        "\x1b_Ga=f,f=32,i={},z={},q=2,m={};{}\x1b\\",
                        image_id, gap, more, chunk_str
                    )?;
                } else {
                    // Continuation chunks of subsequent frames
                    write!(out, "\x1b_Gm={};{}\x1b\\", more, chunk_str)?;
                }
            }
        }

        // Start looping animation: s=3 = loop infinitely, v=1 = start playing
        write!(out, "\x1b_Ga=a,i={},s=3,v=1,q=2;\x1b\\", image_id)?;

        self.uploaded.insert(image_id);
        Ok(())
    }

    /// Place the animation at a terminal cell position.
    /// Deletes the previous placement first to avoid ghost images.
    pub fn place(&mut self, image_id: u32, pos: TermPos, out: &mut impl Write) -> Result<()> {
        let col = pos.col();
        let row = pos.row();

        let same_pos = self.last_pos == Some((col, row));
        let same_img = self.last_image_id == Some(image_id);

        // Skip re-place if nothing changed (saves escape sequence bandwidth)
        if same_pos && same_img {
            return Ok(());
        }

        // Delete all existing placements before re-placing at new position
        write!(out, "\x1b_Ga=d,d=a,q=2;\x1b\\")?;

        // Move cursor to target cell (1-indexed)
        write!(out, "\x1b[{};{}H", row + 1, col + 1)?;

        // Place the animation (Kitty auto-advances frames via the loop we set up)
        write!(out, "\x1b_Ga=p,i={},q=2;\x1b\\", image_id)?;

        self.last_image_id = Some(image_id);
        self.last_pos = Some((col, row));

        out.flush()?;
        Ok(())
    }

    pub fn on_resize(&mut self) {
        // Terminal cleared on resize — re-upload and re-place everything
        self.uploaded.clear();
        self.last_image_id = None;
        self.last_pos = None;
    }
}

fn scale_to_target(img: &RgbaImage) -> RgbaImage {
    if img.width() == TARGET_PX && img.height() == TARGET_PX {
        return img.clone();
    }
    image::imageops::resize(img, TARGET_PX, TARGET_PX, image::imageops::FilterType::Nearest)
}

/// Stable image ID from species + shiny + anim state index.
/// Each (species, shiny, state) combo gets its own Kitty animation.
pub fn image_id(species_id: u16, shiny: bool, state_idx: u8) -> u32 {
    let mut h: u32 = species_id as u32;
    h = h.wrapping_mul(31).wrapping_add(shiny as u32);
    h = h.wrapping_mul(31).wrapping_add(state_idx as u32);
    h.max(1)
}
