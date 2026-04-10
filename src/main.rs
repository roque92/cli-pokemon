use anyhow::Result;
use clap::Parser;

use petcli::{
    cli::{commands, Cli, Commands},
    config,
    renderer,
    state::{decay, persist},
    tui::{app::App, run},
};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = config::load();

    // Load state and apply time-based decay since last session
    let mut party = persist::load()?;
    for slot in party.slots.iter_mut().flatten() {
        decay::apply(slot);
    }

    match cli.command {
        None => {
            let renderer_kind = renderer::detect();
            let mut app = App::new(party, renderer_kind);
            app.compact = cli.compact || cfg.display.compact;
            run::run(app).await?;
        }
        Some(Commands::Catch { shiny }) => {
            let shiny_rate = cfg.game.shiny_rate;
            commands::catch(&mut party, shiny, shiny_rate)?;
        }
        Some(Commands::Status) => {
            commands::status(&party);
        }
        Some(Commands::Feed) => {
            commands::feed(&mut party)?;
        }
        Some(Commands::Pet) => {
            commands::pet_cmd(&mut party)?;
        }
        Some(Commands::Party) => {
            commands::list_party(&party);
        }
        Some(Commands::Inventory) => {
            commands::inventory(&party);
        }
        Some(Commands::Config) => {
            let path = config::init_default()?;
            println!("Config file: {}", path.display());
            println!("Edit it to customise shiny rate, decay speed, compact mode, etc.");
        }
    }

    Ok(())
}
