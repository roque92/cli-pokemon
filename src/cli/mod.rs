pub mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "petcli",
    about = "Animated Pokemon pets living in your terminal",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Internal: run as background daemon with the given renderer
    #[arg(long, hide = true, value_name = "RENDERER")]
    pub daemon: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Set which Pokemon appears (by name or Pokedex number)
    Set {
        /// Pokemon name or number (e.g. "pikachu" or "25")
        pokemon: String,
        /// Force shiny variant
        #[arg(long)]
        shiny: bool,
    },
    /// Show current Pokemon
    Status,
    /// Add another Pokemon (up to 6 walk at once)
    Add {
        /// Pokemon name or number (leave empty to open the picker)
        pokemon: Option<String>,
        /// Force shiny variant
        #[arg(long)]
        shiny: bool,
    },
    /// List all available Pokemon
    Pokemon,
    /// Remove all pets from your terminal
    Stop,
}
