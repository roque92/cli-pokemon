use std::{
    io::{self, Write},
    time::{Duration, Instant},
};

use anyhow::Result;
use crossterm::{
    event,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

use crate::{
    renderer::{kitty::image_id, RendererKind},
    sprite,
    state::{decay, persist},
};

use super::{app::App, draw, draw_compact, input};

const TICK_MS: u64 = 16; // ~60fps

pub async fn run(mut app: App) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (sprite_tx, mut sprite_rx) = mpsc::channel(4);

    // Track which species is currently loaded to detect hatch/evolution changes
    let mut loaded_species: Option<(u16, bool)> = None;

    let mut last_tick = Instant::now();

    loop {
        // Check for loaded sprite
        if let Ok(sheet) = sprite_rx.try_recv() {
            app.on_sprite_loaded(sheet);
        }

        // Kick off sprite load whenever the active pet's species/shiny changes
        // (covers initial load, hatch, evolution, party cycle)
        let current_species = app
            .party
            .active_pet()
            .filter(|p| p.is_alive())
            .map(|p| (p.species_id, p.shiny));

        if current_species != loaded_species && !app.loading_sprite {
            if let Some((species_id, shiny)) = current_species {
                app.loading_sprite = true;
                loaded_species = Some((species_id, shiny));
                let tx = sprite_tx.clone();
                tokio::spawn(async move {
                    match sprite::load_sheet(species_id, shiny).await {
                        Ok(sheet) => { let _ = tx.send(sheet).await; }
                        Err(e) => tracing::warn!("sprite load failed: {}", e),
                    }
                });
            } else {
                loaded_species = None;
            }
        }

        let delta_ms = last_tick.elapsed().as_millis().min(500) as u32;
        last_tick = Instant::now();

        // Apply decay to all living party members
        for slot in app.party.slots.iter_mut().flatten() {
            decay::apply_tick(slot, delta_ms);
        }

        // Tick animation + walk for active pet
        if let Some(sheet) = &app.sheet {
            if let Some(stats) = app.party.active_pet().map(|p| p.stats.clone()) {
                app.ticker.tick(delta_ms, sheet, &stats, &mut app.rng);

                let bounds = app.play_bounds;
                let mut anim_state = app.ticker.current_state;
                if let Some(walk) = &mut app.walk {
                    walk.step(delta_ms, bounds, &mut anim_state, &mut app.rng);
                }
            }
        }

        // Draw TUI (compact or full)
        if app.compact {
            terminal.draw(|f| draw_compact::draw(f, &mut app))?;
        } else {
            terminal.draw(|f| draw::draw(f, &mut app))?;
        }

        // Post-draw: Kitty animation placement
        if app.renderer_kind == RendererKind::Kitty {
            if let Some(sheet) = &app.sheet {
                if let (Some(walk), Some(pet)) = (&app.walk, app.party.active_pet()) {
                    let anim_state = app.ticker.current_state;
                    let frames = sheet.frames_for(anim_state);
                    if !frames.is_empty() {
                        let img_id = image_id(pet.species_id, pet.shiny, anim_state as u8);
                        let pos = walk.pos;
                        let mut out = io::stdout();
                        let _ = app.kitty.upload_animation(img_id, frames, &mut out);
                        let _ = app.kitty.place(img_id, pos, &mut out);
                        let _ = out.flush();
                    }
                }
            }
        }

        // Poll for input
        if event::poll(Duration::from_millis(TICK_MS))? {
            let ev = event::read()?;
            if let Some(tui_event) = input::map_event(ev) {
                app.handle_event(tui_event);
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Clean up Kitty images before exit
    {
        let mut out = io::stdout();
        let _ = write!(out, "\x1b_Ga=d,d=a,q=2;\x1b\\");
        let _ = out.flush();
    }

    persist::save(&app.party)?;
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
