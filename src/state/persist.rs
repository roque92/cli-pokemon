use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;

use super::party::Party;

fn data_dir() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("", "", "petcli")
        .context("could not determine data directory")?;
    let path = dirs.data_dir().to_path_buf();
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

pub fn state_path() -> Result<PathBuf> {
    Ok(data_dir()?.join("state.json"))
}

pub fn sprite_dir() -> Result<PathBuf> {
    let dir = data_dir()?.join("sprites");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn load() -> Result<Party> {
    let path = state_path()?;
    if !path.exists() {
        return Ok(Party::new());
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("reading state from {}", path.display()))?;
    serde_json::from_str(&raw).context("parsing state.json")
}

pub fn save(party: &Party) -> Result<()> {
    let path = state_path()?;
    let raw = serde_json::to_string_pretty(party)?;
    std::fs::write(&path, raw)
        .with_context(|| format!("writing state to {}", path.display()))
}
