use std::path::PathBuf;

use anyhow::Result;

use crate::error::PetcliError;
use crate::state::persist::sprite_dir;

const POKEAPI_BASE: &str =
    "https://raw.githubusercontent.com/PokeAPI/sprites/master/sprites/pokemon";

fn animated_url(species_id: u16, shiny: bool) -> String {
    let base = format!("{}/versions/generation-v/black-white/animated", POKEAPI_BASE);
    if shiny {
        format!("{}/shiny/{}.gif", base, species_id)
    } else {
        format!("{}/{}.gif", base, species_id)
    }
}

fn static_url(species_id: u16, shiny: bool) -> String {
    if shiny {
        format!("{}/shiny/{}.png", POKEAPI_BASE, species_id)
    } else {
        format!("{}/{}.png", POKEAPI_BASE, species_id)
    }
}

fn animated_cache_path(species_id: u16, shiny: bool) -> Result<PathBuf> {
    let dir = sprite_dir()?.join("gen5-animated");
    std::fs::create_dir_all(&dir)?;
    let name = if shiny {
        format!("{}-shiny.gif", species_id)
    } else {
        format!("{}.gif", species_id)
    };
    Ok(dir.join(name))
}

fn static_cache_path(species_id: u16, shiny: bool) -> Result<PathBuf> {
    let dir = sprite_dir()?.join("static");
    std::fs::create_dir_all(&dir)?;
    let name = if shiny {
        format!("{}-shiny.png", species_id)
    } else {
        format!("{}.png", species_id)
    };
    Ok(dir.join(name))
}

/// Returns cached animated GIF bytes, downloading if needed.
/// Falls back to static PNG if animated GIF is unavailable.
pub async fn fetch_sprite(species_id: u16, shiny: bool) -> Result<(Vec<u8>, bool)> {
    if std::env::var("PETCLI_OFFLINE").is_ok() {
        return Err(PetcliError::Offline.into());
    }

    // Try animated first
    let anim_path = animated_cache_path(species_id, shiny)?;
    if anim_path.exists() {
        let bytes = std::fs::read(&anim_path)?;
        return Ok((bytes, true));
    }

    let client = reqwest::Client::new();
    let url = animated_url(species_id, shiny);
    let resp = client.get(&url).send().await;

    if let Ok(r) = resp {
        if r.status().is_success() {
            let bytes = r.bytes().await?.to_vec();
            std::fs::write(&anim_path, &bytes)?;
            return Ok((bytes, true));
        }
    }

    // Fall back to static PNG
    let static_path = static_cache_path(species_id, shiny)?;
    if static_path.exists() {
        let bytes = std::fs::read(&static_path)?;
        return Ok((bytes, false));
    }

    let url = static_url(species_id, shiny);
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| PetcliError::SpriteFetch(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(PetcliError::SpriteFetch(format!(
            "HTTP {} for species {}",
            resp.status(),
            species_id
        ))
        .into());
    }

    let bytes = resp.bytes().await?.to_vec();
    std::fs::write(&static_path, &bytes)?;
    Ok((bytes, false))
}
