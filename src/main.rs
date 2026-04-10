use anyhow::Result;
use clap::Parser;

use petcli::{
    cli::{commands, Cli, Commands},
    renderer::{self, RendererKind},
    state::persist,
    tui::{daemon, picker, run},
};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Hidden daemon mode: spawned by `run::run()`, animates the pet in bg
    if let Some(renderer_flag) = cli.daemon {
        let kind = if renderer_flag == "kitty" {
            RendererKind::Kitty
        } else {
            RendererKind::Unicode
        };
        return daemon::run(kind).await;
    }

    let mut party = persist::load()?;

    match cli.command {
        None => {
            let renderer_kind = renderer::detect();
            run::run(party, renderer_kind).await?;
        }
        Some(Commands::Set { pokemon, shiny }) => {
            commands::set_pokemon(&mut party, &pokemon, shiny)?;
            persist::save(&party)?;
        }
        Some(Commands::Add { pokemon, shiny }) => {
            if let Some(name) = pokemon {
                // Direct add by name/number
                commands::add_pokemon(&mut party, &name, shiny)?;
                persist::save(&party)?;
            } else {
                // Open TUI picker
                let renderer_kind = renderer::detect();
                picker::run(&mut party, renderer_kind).await?;
                persist::save(&party)?;
            }
        }
        Some(Commands::Pokemon) => {
            commands::list_pokemon();
        }
        Some(Commands::Status) => {
            commands::status(&party);
        }
        Some(Commands::Stop) => {
            run::stop()?;
        }
    }

    Ok(())
}
