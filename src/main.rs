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
        let kind = match renderer_flag.as_str() {
            "kitty"  => RendererKind::Kitty,
            "iterm2" => RendererKind::Iterm2,
            _        => RendererKind::Unicode,
        };
        return daemon::run(kind).await;
    }

    let mut party = persist::load()?;
    let renderer_kind = renderer::detect();

    match cli.command {
        // No subcommand → start/check the daemon
        None => {
            run::run(party, renderer_kind).await?;
        }

        Some(Commands::Set { pokemon, shiny }) => {
            commands::set_pokemon(&mut party, &pokemon, shiny)?;
            persist::save(&party)?;
            run::restart_if_running(&party, renderer_kind).await?;
        }

        Some(Commands::Add { pokemon, shiny }) => {
            commands::add_pokemon(&mut party, &pokemon, shiny)?;
            persist::save(&party)?;
            run::restart_if_running(&party, renderer_kind).await?;
        }

        Some(Commands::Pokemon) => {
            // Open TUI picker; user may add one or more Pokemon
            picker::run(&mut party, renderer_kind).await?;
            persist::save(&party)?;
            run::restart_if_running(&party, renderer_kind).await?;
        }

        Some(Commands::Remove { number }) => {
            commands::remove_pokemon(&mut party, number)?;
            persist::save(&party)?;
            run::restart_if_running(&party, renderer_kind).await?;
        }

        Some(Commands::Reset) => {
            commands::reset_party(&mut party);
            persist::save(&party)?;
            // Stop the daemon since there's nothing left to show
            run::stop()?;
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
