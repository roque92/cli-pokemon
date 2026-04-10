use std::path::PathBuf;

use anyhow::Result;

use crate::error::PetcliError;
use crate::state::persist::sprite_dir;

// vscode-pokemon sprite repo (proper idle + walk GIFs, 8fps, 48×48)
const VSPOKE_BASE: &str =
    "https://raw.githubusercontent.com/jakobhoeg/vscode-pokemon/main/media";

// PokeAPI animated Gen 5 sprites (fallback)
const POKEAPI_ANIMATED: &str =
    "https://raw.githubusercontent.com/PokeAPI/sprites/master/sprites/pokemon/versions/generation-v/black-white/animated";

const POKEAPI_STATIC: &str =
    "https://raw.githubusercontent.com/PokeAPI/sprites/master/sprites/pokemon";

/// Normalize a Pokemon name to vscode-pokemon folder naming:
/// lowercase, spaces→hyphens, ♀→f, ♂→m, strip .'
fn normalize_name(name: &str) -> String {
    name.to_lowercase()
        .replace('♀', "f")
        .replace('♂', "m")
        .replace(['.', '\''], "")
        .replace(' ', "-")
}

fn gen_folder(species_id: u16) -> &'static str {
    match species_id {
        1..=151   => "gen1",
        152..=251 => "gen2",
        252..=386 => "gen3",
        387..=493 => "gen4",
        _         => "gen1",
    }
}

fn vspoke_url(name: &str, species_id: u16, anim: &str) -> String {
    format!(
        "{}/{}/{}/default_{}_8fps.gif",
        VSPOKE_BASE,
        gen_folder(species_id),
        normalize_name(name),
        anim
    )
}

fn cache_path(subfolder: &str, filename: &str) -> Result<PathBuf> {
    let dir = sprite_dir()?.join(subfolder);
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join(filename))
}

async fn get_bytes(client: &reqwest::Client, url: &str) -> Option<Vec<u8>> {
    let resp = client.get(url).send().await.ok()?;
    if resp.status().is_success() {
        resp.bytes().await.ok().map(|b| b.to_vec())
    } else {
        None
    }
}

/// Returns `(idle_bytes, walk_bytes_opt, is_gif)`.
///
/// Strategy (in order):
/// 1. vscode-pokemon repo — separate idle + walk GIFs (Gen 1–4, non-shiny)
/// 2. PokeAPI Gen 5 animated GIF (shiny or Gen 5+) — used for both idle and walk
/// 3. PokeAPI static PNG — single frame fallback
pub async fn fetch_sprite_pair(
    species_id: u16,
    shiny: bool,
    name: &str,
) -> Result<(Vec<u8>, Option<Vec<u8>>, bool)> {
    if std::env::var("PETCLI_OFFLINE").is_ok() {
        return Err(PetcliError::Offline.into());
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    // 1. Try vscode-pokemon (Gen 1–4, non-shiny only)
    if !shiny && species_id <= 493 {
        let idle_cache = cache_path("vspoke", &format!("{}-idle.gif", species_id))?;
        let walk_cache = cache_path("vspoke", &format!("{}-walk.gif", species_id))?;

        if idle_cache.exists() {
            let idle = std::fs::read(&idle_cache)?;
            let walk = if walk_cache.exists() {
                Some(std::fs::read(&walk_cache)?)
            } else {
                None
            };
            return Ok((idle, walk, true));
        }

        let idle_url = vspoke_url(name, species_id, "idle");
        let walk_url = vspoke_url(name, species_id, "walk");

        if let Some(idle) = get_bytes(&client, &idle_url).await {
            std::fs::write(&idle_cache, &idle)?;
            let walk = get_bytes(&client, &walk_url).await.map(|w| {
                let _ = std::fs::write(&walk_cache, &w);
                w
            });
            return Ok((idle, walk, true));
        }
    }

    // 2. PokeAPI animated Gen 5 GIF
    let anim_suffix = if shiny { "-shiny" } else { "" };
    let anim_cache = cache_path(
        "gen5-animated",
        &format!("{}{}.gif", species_id, anim_suffix),
    )?;

    if anim_cache.exists() {
        let bytes = std::fs::read(&anim_cache)?;
        return Ok((bytes, None, true));
    }

    let anim_url = if shiny {
        format!("{}/shiny/{}.gif", POKEAPI_ANIMATED, species_id)
    } else {
        format!("{}/{}.gif", POKEAPI_ANIMATED, species_id)
    };

    if let Some(bytes) = get_bytes(&client, &anim_url).await {
        std::fs::write(&anim_cache, &bytes)?;
        return Ok((bytes, None, true));
    }

    // 3. Static PNG fallback
    let png_cache = cache_path(
        "static",
        &format!("{}{}.png", species_id, anim_suffix),
    )?;

    if png_cache.exists() {
        let bytes = std::fs::read(&png_cache)?;
        return Ok((bytes, None, false));
    }

    let png_url = if shiny {
        format!("{}/shiny/{}.png", POKEAPI_STATIC, species_id)
    } else {
        format!("{}/{}.png", POKEAPI_STATIC, species_id)
    };

    let bytes = get_bytes(&client, &png_url)
        .await
        .ok_or_else(|| PetcliError::SpriteFetch(format!("no sprite for species {}", species_id)))?;

    std::fs::write(&png_cache, &bytes)?;
    Ok((bytes, None, false))
}
