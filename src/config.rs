use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub display: DisplayConfig,
    pub game: GameConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Launch in compact mode by default
    pub compact: bool,
    /// Kitty sprite size in pixels (power of 2 recommended)
    pub sprite_size: u32,
    /// Show shiny sparkle indicator in play area
    pub show_shiny_indicator: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    /// 1-in-N chance of shiny on catch (default 8192)
    pub shiny_rate: u32,
    /// Multiplier for stat decay speed (1.0 = normal, 0.5 = slower, 2.0 = faster)
    pub decay_rate: f32,
    /// Coins earned per feed interaction
    pub coins_per_feed: u32,
    /// Coins earned per pet interaction
    pub coins_per_pet: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display: DisplayConfig {
                compact: false,
                sprite_size: 96,
                show_shiny_indicator: true,
            },
            game: GameConfig {
                shiny_rate: 8192,
                decay_rate: 1.0,
                coins_per_feed: 2,
                coins_per_pet: 5,
            },
        }
    }
}

fn config_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("", "", "petcli")
        .context("could not determine config directory")?;
    let dir = dirs.config_dir().to_path_buf();
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("config.toml"))
}

pub fn load() -> Config {
    load_inner().unwrap_or_default()
}

fn load_inner() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let raw = std::fs::read_to_string(&path)?;
    toml::from_str(&raw).context("parsing config.toml")
}

pub fn save(config: &Config) -> Result<()> {
    let path = config_path()?;
    let raw = toml::to_string_pretty(config)?;
    std::fs::write(path, raw)?;
    Ok(())
}

pub fn init_default() -> Result<PathBuf> {
    let path = config_path()?;
    if !path.exists() {
        let raw = toml::to_string_pretty(&Config::default())?;
        std::fs::write(&path, raw)?;
    }
    Ok(path)
}
