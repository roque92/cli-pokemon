pub mod fetch;
pub mod gif;
pub mod sheet;

use anyhow::Result;

use self::{
    fetch::fetch_sprite_pair,
    gif::{decode_gif, decode_png},
    sheet::SpriteSheet,
};

/// Load a SpriteSheet with separate idle and walk animations.
/// Downloads from vscode-pokemon repo (idle+walk GIFs) with PokeAPI fallback.
pub async fn load_sheet(species_id: u16, shiny: bool) -> Result<SpriteSheet> {
    let name = crate::state::species::name(species_id);
    let (idle_bytes, walk_bytes, is_gif) =
        fetch_sprite_pair(species_id, shiny, &name).await?;

    if is_gif {
        let idle = decode_gif(&idle_bytes)?;
        let walk = if let Some(wb) = walk_bytes {
            decode_gif(&wb).unwrap_or_default()
        } else {
            Vec::new()
        };
        Ok(SpriteSheet::new(species_id, shiny, idle, walk))
    } else {
        let frames = decode_png(&idle_bytes)?;
        Ok(SpriteSheet::from_single(species_id, shiny, frames))
    }
}
