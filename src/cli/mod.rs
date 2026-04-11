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
    /// Set which Pokemon appears, replacing the whole party
    Set {
        /// Pokemon name or number (e.g. "pikachu" or "25")
        pokemon: String,
        /// Force shiny variant
        #[arg(long)]
        shiny: bool,
    },
    /// Show all Pokemon currently in your party (with slot numbers)
    Status,
    /// Add another Pokemon by name or number (up to 6 walk at once)
    Add {
        /// Pokemon name or number (e.g. "pikachu" or "25")
        pokemon: String,
        /// Force shiny variant
        #[arg(long)]
        shiny: bool,
    },
    /// Browse all 493 Pokemon with live sprite preview and add to party
    Pokemon,
    /// Remove a Pokemon from your party by its slot number (see `petcli status`)
    Remove {
        /// Slot number shown in `petcli status` (e.g. 2 to remove the 2nd Pokemon)
        number: usize,
    },
    /// Wipe all Pokemon from your party (does not stop the daemon)
    Reset,
    /// Stop the walking daemon and clear all pets from the terminal
    Stop,
}
