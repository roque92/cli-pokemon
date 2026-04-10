use anyhow::{Context, Result};
use image::RgbaImage;

use crate::error::PetcliError;

#[derive(Debug, Clone)]
pub struct DecodedFrame {
    pub pixels: RgbaImage,
    /// Frame display duration in milliseconds
    pub delay_ms: u32,
}

/// Decode a GIF byte slice into a sequence of fully-composited RGBA frames.
/// Uses gif-dispose to correctly handle all GIF disposal methods (critical for
/// Gen 5 Pokemon sprites which use RestoreToBackground between frames).
pub fn decode_gif(data: &[u8]) -> Result<Vec<DecodedFrame>> {
    use gif::DecodeOptions;
    use gif_dispose::Screen;

    let mut opts = DecodeOptions::new();
    opts.set_color_output(gif::ColorOutput::Indexed);

    let mut decoder = opts
        .read_info(std::io::Cursor::new(data))
        .map_err(|e: gif::DecodingError| PetcliError::SpriteDecode(e.to_string()))?;

    let mut screen = Screen::new_decoder(&decoder);
    let mut frames = Vec::new();

    loop {
        match decoder.read_next_frame() {
            Ok(Some(frame)) => {
                screen
                    .blit_frame(frame)
                    .map_err(|e| PetcliError::SpriteDecode(e.to_string()))?;

                let px = screen.pixels_rgba();
                let width = px.width() as u32;
                let height = px.height() as u32;
                let mut img = RgbaImage::new(width, height);

                for y in 0..height {
                    let row = &px[y as usize];
                    for x in 0..width {
                        let p = row[x as usize];
                        img.put_pixel(x, y, image::Rgba([p.r, p.g, p.b, p.a]));
                    }
                }

                // GIF delay is in centiseconds → convert to ms
                let delay_ms = (frame.delay as u32) * 10;
                // Clamp: some sprites have 0ms delay which would cause infinite loop
                let delay_ms = delay_ms.max(50);

                frames.push(DecodedFrame { pixels: img, delay_ms });
            }
            Ok(None) => break,
            Err(e) => return Err(PetcliError::SpriteDecode(e.to_string()).into()),
        }
    }

    if frames.is_empty() {
        return Err(PetcliError::SpriteDecode("GIF contained no frames".into()).into());
    }

    Ok(frames)
}

/// Decode a static PNG into a single-frame "animation" at 200ms tick.
pub fn decode_png(data: &[u8]) -> Result<Vec<DecodedFrame>> {
    let img = image::load_from_memory(data)
        .context("decoding PNG sprite")?
        .to_rgba8();

    Ok(vec![DecodedFrame { pixels: img, delay_ms: 200 }])
}
