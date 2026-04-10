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

    /// Compact mode: minimal pane for tmux/zellij (no sidebar)
    #[arg(long, short = 'c', global = true)]
    pub compact: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Catch a new random Pokemon egg
    Catch {
        /// Force a shiny (debug)
        #[arg(long, hide = true)]
        shiny: bool,
    },
    /// Show party status
    Status,
    /// Feed your active Pokemon
    Feed,
    /// Pet your active Pokemon
    Pet,
    /// List and manage your party
    Party,
    /// Show inventory and wallet
    Inventory,
    /// Print and initialise config file path
    Config,
}
