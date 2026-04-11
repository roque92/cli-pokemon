//! Interactive Pokemon picker TUI — opened by `petcli pokemon`.
//!
//! Layout:
//!   ┌── Gen tabs ──────────────────────────────────────────────┐
//!   │  All  Gen1  Gen2  Gen3  Gen4                             │
//!   ├────────────────────────────┬─────────────────────────────┤
//!   │ Search: _                  │                             │
//!   ├────────────────────────────┤   [Sprite preview]          │
//!   │ >  25. Pikachu             │                             │
//!   │    26. Raichu              │   Pikachu  #025             │
//!   │    ...                     │   [Enter] Add  [s] Shiny    │
//!   ├────────────────────────────┴─────────────────────────────┤
//!   │  ↑↓ nav  / search  Tab gen  Enter add  s shiny  q quit  │
//!   └──────────────────────────────────────────────────────────┘

use std::{
    collections::HashSet,
    io::{self, Write},
    time::Duration,
};

use anyhow::Result;
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use tokio::sync::mpsc;

use crate::{
    renderer::{
        kitty::{image_id, KittyRenderer},
        unicode,
        RendererKind,
    },
    sprite::{self, sheet::SpriteSheet},
    state::{
        party::Party,
        pet::{Pet, PetStatus},
        pokedex::{gen_of, POKEMON},
    },
    tui::run as daemon_run,
};

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum GenTab { All, Gen1, Gen2, Gen3, Gen4 }

impl GenTab {
    const ALL: &'static [GenTab] = &[
        GenTab::All, GenTab::Gen1, GenTab::Gen2, GenTab::Gen3, GenTab::Gen4,
    ];
    fn label(self) -> &'static str {
        match self {
            Self::All  => "All",
            Self::Gen1 => "Gen1",
            Self::Gen2 => "Gen2",
            Self::Gen3 => "Gen3",
            Self::Gen4 => "Gen4",
        }
    }
    fn matches(self, id: u16) -> bool {
        match self {
            Self::All  => true,
            Self::Gen1 => gen_of(id) == 1,
            Self::Gen2 => gen_of(id) == 2,
            Self::Gen3 => gen_of(id) == 3,
            Self::Gen4 => gen_of(id) == 4,
        }
    }
}

/// Sprite loading state — always retains the previously-loaded sheet so the
/// preview panel never goes blank while a new sprite is being fetched.
struct SpriteCache {
    /// The sprite currently shown in the preview (may be stale while loading).
    current: Option<(SpriteSheet, usize)>,
    /// Species ID being fetched right now, or None if idle.
    loading: Option<u16>,
    /// Set when the last fetch failed and no current sprite exists.
    failed: bool,
}

impl SpriteCache {
    fn new() -> Self {
        Self { current: None, loading: None, failed: false }
    }

    fn is_loading(&self) -> bool { self.loading.is_some() }

    fn is_ready_for(&self, id: u16, shiny: bool) -> bool {
        self.current.as_ref()
            .map(|(s, _)| s.species_id == id && s.shiny == shiny)
            .unwrap_or(false)
    }

    fn set_loaded(&mut self, id: u16, shiny: bool, result: Result<SpriteSheet, anyhow::Error>) {
        if self.loading == Some(id) {
            self.loading = None;
        }
        match result {
            Ok(sheet) if sheet.species_id == id && sheet.shiny == shiny => {
                self.current = Some((sheet, 0));
                self.failed = false;
            }
            Ok(_) => {} // species changed before load finished — discard
            Err(_) => {
                // Only mark failed if we have nothing to show
                if self.current.is_none() {
                    self.failed = true;
                }
            }
        }
    }

    fn tick_frame(&mut self) {
        if let Some((_, frame_idx)) = &mut self.current {
            *frame_idx = frame_idx.wrapping_add(1);
        }
    }

    fn frame_idx(&self) -> usize {
        self.current.as_ref().map(|(_, i)| *i).unwrap_or(0)
    }

    fn sheet(&self) -> Option<&SpriteSheet> {
        self.current.as_ref().map(|(s, _)| s)
    }
}

struct PickerState {
    // List
    filtered: Vec<usize>,     // indices into POKEMON
    list_state: ListState,
    gen_tab: GenTab,
    search: String,
    search_mode: bool,
    shiny: bool,
    // Sprite preview
    sprite: SpriteCache,
    sprite_tx: mpsc::Sender<(u16, bool, Result<SpriteSheet>)>,
    sprite_rx: mpsc::Receiver<(u16, bool, Result<SpriteSheet>)>,
    kitty: KittyRenderer,
    // Party context (to show which are already added)
    party_ids: HashSet<u16>,
    // Result
    pub added: Option<(u16, bool)>,
}

impl PickerState {
    fn new(party: &Party) -> Self {
        let (tx, rx) = mpsc::channel(8);
        let party_ids: HashSet<u16> = party
            .slots.iter().flatten()
            .filter(|p| matches!(p.status, PetStatus::Alive))
            .map(|p| p.species_id)
            .collect();

        let mut s = Self {
            filtered: Vec::new(),
            list_state: ListState::default(),
            gen_tab: GenTab::All,
            search: String::new(),
            search_mode: false,
            shiny: false,
            sprite: SpriteCache::new(),
            sprite_tx: tx,
            sprite_rx: rx,
            kitty: KittyRenderer::new(),
            party_ids,
            added: None,
        };
        s.rebuild_filter();
        s
    }

    fn rebuild_filter(&mut self) {
        let q = self.search.to_lowercase();
        self.filtered = POKEMON.iter().enumerate()
            .filter(|(_, (id, name))| {
                self.gen_tab.matches(*id)
                    && (q.is_empty() || name.to_lowercase().contains(&q))
            })
            .map(|(i, _)| i)
            .collect();

        // Keep selection in range
        let len = self.filtered.len();
        let sel = self.list_state.selected().unwrap_or(0).min(len.saturating_sub(1));
        if len > 0 {
            self.list_state.select(Some(sel));
        } else {
            self.list_state.select(None);
        }

        self.trigger_sprite_load();
    }

    fn selected_pokemon(&self) -> Option<(u16, &'static str)> {
        let idx = self.list_state.selected()?;
        let pidx = *self.filtered.get(idx)?;
        let (id, name) = POKEMON[pidx];
        Some((id, name))
    }

    fn trigger_sprite_load(&mut self) {
        let Some((id, _)) = self.selected_pokemon() else { return };
        // Skip if we already have this species+shiny loaded or are fetching it
        if self.sprite.is_ready_for(id, self.shiny) { return; }
        if self.sprite.loading == Some(id) { return; }

        self.sprite.loading = Some(id);
        self.sprite.failed = false;
        let tx = self.sprite_tx.clone();
        let shiny = self.shiny;
        tokio::spawn(async move {
            let result = sprite::load_sheet(id, shiny).await;
            let _ = tx.send((id, shiny, result)).await;
        });
    }

    fn move_up(&mut self) {
        let len = self.filtered.len();
        if len == 0 { return; }
        let sel = self.list_state.selected().unwrap_or(0);
        self.list_state.select(Some(if sel == 0 { len - 1 } else { sel - 1 }));
        self.trigger_sprite_load();
    }

    fn move_down(&mut self) {
        let len = self.filtered.len();
        if len == 0 { return; }
        let sel = self.list_state.selected().unwrap_or(0);
        self.list_state.select(Some((sel + 1) % len));
        self.trigger_sprite_load();
    }

    fn set_gen(&mut self, tab: GenTab) {
        self.gen_tab = tab;
        self.list_state.select(Some(0));
        self.rebuild_filter();
    }

    fn toggle_shiny(&mut self) {
        self.shiny = !self.shiny;
        // Reset only the loading state; keep current sprite shown until shiny loads
        self.sprite.loading = None;
        self.trigger_sprite_load();
    }

    fn poll_sprite(&mut self) {
        while let Ok((id, shiny, result)) = self.sprite_rx.try_recv() {
            // Only apply if still relevant to current selection
            if let Some((cur_id, _)) = self.selected_pokemon() {
                if cur_id == id && shiny == self.shiny {
                    self.sprite.set_loaded(id, shiny, result);
                }
            }
        }
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub async fn run(party: &mut Party, renderer_kind: RendererKind) -> Result<()> {
    // Freeze the walking daemon so it doesn't write to the TTY while the
    // picker TUI is open (would cause flashing in the preview panel).
    daemon_run::pause_daemon();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = PickerState::new(party);
    state.trigger_sprite_load();

    let mut preview_rect = Rect::default();

    loop {
        // Poll completed sprite loads
        state.poll_sprite();

        // Advance sprite animation frame
        state.sprite.tick_frame();

        // Draw TUI (ratatui frame)
        terminal.draw(|f| {
            preview_rect = draw_ui(f, &mut state);
        })?;

        // Post-draw: render sprite directly over the preview panel (bypasses ratatui)
        render_sprite_preview(&mut state, renderer_kind, preview_rect)?;
        io::stdout().flush()?;

        // Handle input (16ms timeout → ~60fps redraw so sprite stays visible)
        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(k) => {
                    if state.search_mode {
                        match k.code {
                            KeyCode::Esc   => { state.search_mode = false; }
                            KeyCode::Enter => { state.search_mode = false; }
                            KeyCode::Backspace => { state.search.pop(); state.rebuild_filter(); }
                            KeyCode::Char(c)   => { state.search.push(c); state.rebuild_filter(); }
                            _ => {}
                        }
                    } else {
                        match k.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => break,
                            KeyCode::Up   | KeyCode::Char('k') => state.move_up(),
                            KeyCode::Down | KeyCode::Char('j') => state.move_down(),
                            KeyCode::Char('/') => { state.search_mode = true; }
                            KeyCode::Char('s') => { state.toggle_shiny(); }
                            KeyCode::Tab => {
                                let tabs = GenTab::ALL;
                                let cur = tabs.iter().position(|t| *t == state.gen_tab).unwrap_or(0);
                                state.set_gen(tabs[(cur + 1) % tabs.len()]);
                            }
                            KeyCode::Char('1') => state.set_gen(GenTab::Gen1),
                            KeyCode::Char('2') => state.set_gen(GenTab::Gen2),
                            KeyCode::Char('3') => state.set_gen(GenTab::Gen3),
                            KeyCode::Char('4') => state.set_gen(GenTab::Gen4),
                            KeyCode::Char('a') => state.set_gen(GenTab::All),
                            KeyCode::Enter => {
                                if let Some((id, _name)) = state.selected_pokemon() {
                                    state.added = Some((id, state.shiny));
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Event::Resize(_, _) => {
                    state.kitty.on_resize();
                }
                _ => {}
            }
        }
    }

    // Cleanup Kitty images
    {
        let mut out = io::stdout();
        let _ = write!(out, "\x1b_Ga=d,d=a,q=2;\x1b\\");
        let _ = out.flush();
    }

    execute!(io::stdout(), LeaveAlternateScreen, Show)?;
    disable_raw_mode()?;

    // Unfreeze the daemon now that we're back on the main screen.
    // (restart_if_running in main.rs will handle the full restart if needed;
    //  resume here covers the no-change / cancel case.)
    daemon_run::resume_daemon();

    // Apply selection to party
    if let Some((species_id, shiny)) = state.added {
        use crate::error::PetcliError;
        match party.add_pet(Pet::new_alive(species_id, shiny)) {
            Ok(_) => {
                let name = POKEMON.iter().find(|(id, _)| *id == species_id)
                    .map(|(_, n)| *n).unwrap_or("Pokemon");
                println!("Added {}{}!", name, if shiny { " ✦ (shiny)" } else { "" });
                println!("Run `petcli` to see them walk!");
            }
            Err(PetcliError::PartyFull) => {
                eprintln!("Party is full (6 Pokemon max). Use `petcli set` to replace one.");
            }
            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}

// ── Draw ──────────────────────────────────────────────────────────────────────

/// Draw the TUI and return the Rect where the sprite preview should go.
fn draw_ui(f: &mut Frame, state: &mut PickerState) -> Rect {
    let area = f.area();

    // Outer vertical split: tabs(3) | body(rest) | help(1)
    let outer = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ]).split(area);

    draw_tabs(f, outer[0], state);

    // Body: list(40%) | preview(60%)
    let body = Layout::horizontal([
        Constraint::Percentage(40),
        Constraint::Percentage(60),
    ]).split(outer[1]);

    draw_list(f, body[0], state);
    draw_preview_border(f, body[1], state);
    draw_help(f, outer[2]);

    body[1] // return preview rect for post-draw sprite rendering
}

fn draw_tabs(f: &mut Frame, area: Rect, state: &PickerState) {
    let tabs: Vec<Span> = GenTab::ALL.iter().flat_map(|tab| {
        let active = *tab == state.gen_tab;
        let style = if active {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        vec![Span::styled(format!(" {} ", tab.label()), style), Span::raw("│")]
    }).collect();

    let line = Line::from(tabs);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Pokédex  [Tab] cycle gens  [1-4] jump  [a] all ");
    let inner = block.inner(area);
    f.render_widget(block, area);
    f.render_widget(Paragraph::new(line), inner);
}

fn draw_list(f: &mut Frame, area: Rect, state: &mut PickerState) {
    // Split: search(3) | list(rest)
    let parts = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
    ]).split(area);

    // Search box
    let search_text = if state.search_mode {
        format!(" /{}_", state.search)
    } else if !state.search.is_empty() {
        format!(" /{}", state.search)
    } else {
        " / to search...".to_string()
    };
    let search_style = if state.search_mode {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let search_widget = Paragraph::new(search_text)
        .style(search_style)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(search_widget, parts[0]);

    // Pokemon list
    let items: Vec<ListItem> = state.filtered.iter().map(|&pidx| {
        let (id, name) = POKEMON[pidx];
        let gen = gen_of(id);
        let gen_color = match gen {
            1 => Color::Green,
            2 => Color::Yellow,
            3 => Color::Blue,
            4 => Color::Magenta,
            _ => Color::White,
        };
        let in_party = state.party_ids.contains(&id);
        let marker = if in_party { " ✓" } else { "  " };
        let line = Line::from(vec![
            Span::styled(format!("{:>3}.", id), Style::default().fg(Color::DarkGray)),
            Span::raw(" "),
            Span::styled(format!("{:<14}", name), Style::default().fg(gen_color)),
            Span::styled(marker, Style::default().fg(Color::Green)),
        ]);
        ListItem::new(line)
    }).collect();

    let count = state.filtered.len();
    let title = format!(" {} Pokemon ", count);
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(title),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, parts[1], &mut state.list_state);
}

fn draw_preview_border(f: &mut Frame, area: Rect, state: &PickerState) {
    let (name, id, shiny_marker) = if let Some((id, name)) = state.selected_pokemon() {
        let sm = if state.shiny { " ✦ shiny" } else { "" };
        (name, id, sm)
    } else {
        ("", 0u16, "")
    };

    let title = if id > 0 {
        format!(" #{:0>3} {}{} ", id, name, shiny_marker)
    } else {
        " Preview ".to_string()
    };

    // Loading indicator in title when fetching a new sprite
    let title = if state.sprite.is_loading() && !state.sprite.is_ready_for(id, state.shiny) {
        format!("{} ⟳ ", title.trim_end())
    } else {
        title
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title);

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Status when there's nothing to show yet
    let show_status = state.sprite.sheet().is_none();
    if show_status {
        let status = if state.sprite.failed {
            "  (sprite unavailable)"
        } else if state.sprite.is_loading() {
            "  Loading sprite..."
        } else {
            "  Select a Pokemon"
        };
        let mid = inner.y + inner.height / 2;
        f.render_widget(
            Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            Rect { x: inner.x, y: mid, width: inner.width, height: 1 },
        );
    }

    // Key hints near bottom of inner area
    if inner.height > 2 {
        let hint_row = inner.y + inner.height - 1;
        let hint = Line::from(vec![
            Span::styled(" [Enter]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" Add  "),
            Span::styled("[s]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Shiny  "),
            Span::styled("[q]", Style::default().fg(Color::Red)),
            Span::raw(" Cancel"),
        ]);
        f.render_widget(
            Paragraph::new(hint),
            Rect { x: inner.x, y: hint_row, width: inner.width, height: 1 },
        );
    }
}

fn draw_help(f: &mut Frame, area: Rect) {
    let spans = vec![
        Span::styled(" ↑↓/jk", Style::default().fg(Color::Cyan)),
        Span::raw(" nav  "),
        Span::styled("/", Style::default().fg(Color::Cyan)),
        Span::raw(" search  "),
        Span::styled("Tab/1-4", Style::default().fg(Color::Cyan)),
        Span::raw(" gen  "),
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(" add  "),
        Span::styled("s", Style::default().fg(Color::Yellow)),
        Span::raw(" shiny  "),
        Span::styled("q", Style::default().fg(Color::Red)),
        Span::raw(" quit "),
    ];
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

// ── Sprite rendering ──────────────────────────────────────────────────────────

fn render_sprite_preview(
    state: &mut PickerState,
    renderer_kind: RendererKind,
    panel: Rect,
) -> Result<()> {
    let Some(sheet) = state.sprite.sheet() else { return Ok(()); };
    let frame_idx = state.sprite.frame_idx();

    // Center sprite in the panel (leave room for border + hint line)
    let inner_x = panel.x + 1;
    let inner_y = panel.y + 1;
    let inner_w = panel.width.saturating_sub(2);
    let inner_h = panel.height.saturating_sub(4); // leave space for hint

    match renderer_kind {
        RendererKind::Kitty => {
            let (sprite_cols, sprite_rows) = (6u16, 3u16);
            let col = inner_x + inner_w.saturating_sub(sprite_cols) / 2;
            let row = inner_y + inner_h.saturating_sub(sprite_rows) / 2;

            let frames = sheet.idle_frames();
            if !frames.is_empty() {
                let id = image_id(sheet.species_id, sheet.shiny, false, false);
                let mut out = io::stdout();
                let _ = state.kitty.upload(id, frames, false, &mut out);
                let _ = state.kitty.place(id, col, row, &mut out);
            }
        }
        RendererKind::Iterm2 => {
            use crate::renderer::iterm2;
            use base64::{engine::general_purpose::STANDARD as B64, Engine};
            use image::{imageops, ImageEncoder};

            let frames = sheet.idle_frames();
            if frames.is_empty() { return Ok(()); }
            let frame = &frames[frame_idx % frames.len()];

            let col = inner_x + inner_w.saturating_sub(iterm2::SPRITE_COLS) / 2;
            let row = inner_y + inner_h.saturating_sub(iterm2::SPRITE_ROWS) / 2;

            let img = &frame.pixels;
            let scaled = imageops::resize(img, 48, 48, imageops::FilterType::Nearest);
            let mut buf = Vec::new();
            let enc = image::codecs::png::PngEncoder::new(&mut buf);
            let _ = enc.write_image(scaled.as_raw(), 48, 48, image::ExtendedColorType::Rgba8);
            let encoded = B64.encode(&buf);

            let mut out = io::stdout();
            iterm2::draw_frame(&mut out, &encoded, col, row)?;
        }
        RendererKind::Unicode => {
            let frames = sheet.idle_frames();
            if frames.is_empty() { return Ok(()); }
            let frame = &frames[frame_idx % frames.len()];

            // Use a larger size in the picker (24×12 cells) for visibility
            const PREVIEW_COLS: u16 = 24;
            const PREVIEW_ROWS: u16 = 12;
            let rendered = unicode::render_frame_sized(&frame.pixels, PREVIEW_COLS, PREVIEW_ROWS, false);

            let sprite_rows = rendered.len() as u16;
            let sprite_cols = rendered.first().map(|r| r.len() as u16).unwrap_or(0);
            let col = inner_x + inner_w.saturating_sub(sprite_cols) / 2;
            let row = inner_y + inner_h.saturating_sub(sprite_rows) / 2;

            let mut out = io::stdout();
            unicode::draw_frame(&mut out, &rendered, col, row)?;
        }
    }

    Ok(())
}
