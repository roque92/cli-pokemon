pub mod fetch;
pub mod gif;
pub mod sheet;

use anyhow::Result;

use self::{fetch::fetch_sprite, gif::{decode_gif, decode_png}, sheet::SpriteSheet};

/// Load a SpriteSheet for a species, fetching/caching as needed.
/// Returns None (not Err) while the download is in progress — callers
/// show a placeholder and retry on the next tick.
pub async fn load_sheet(species_id: u16, shiny: bool) -> Result<SpriteSheet> {
    let (bytes, is_animated) = fetch_sprite(species_id, shiny).await?;

    let frames = if is_animated {
        decode_gif(&bytes)?
    } else {
        decode_png(&bytes)?
    };

    Ok(SpriteSheet::from_frames(species_id, shiny, frames))
}
