# petcli

Animated Pokemon pets that live in your terminal. Sprites walk freely across the screen while you work — no new window, no alternate screen, just your current terminal.

```
petcli add        # open picker → choose a Pokemon → it appears immediately
petcli stop       # remove all pets
```

> **The gap:** `krabby` shows static Unicode art. `vscode-pokemon` shows animated pixel sprites — but only inside VS Code. **petcli** brings that same animation to any terminal.

---

## Features

| Feature | Detail |
|---|---|
| Animated sprites | Idle + walk GIFs at 8fps, 48×48px source |
| Two renderers | Kitty graphics protocol (pixel-perfect) · Unicode half-blocks (universal fallback) |
| Full Gen 1–4 | 493 Pokemon via the vscode-pokemon sprite pack |
| Gen 5+ fallback | PokeAPI animated sprites (Gen 5 Black/White) · static PNG otherwise |
| Shiny variants | `--shiny` flag or toggle inside the picker |
| TUI picker | Searchable list with live sprite preview, filterable by generation |
| Multi-pet | Up to 6 pets, each with its own independent 2D bouncing walker |
| Background daemon | Detached process, PID-managed, lives inside your current terminal session |
| Cursor-safe | Cursor position never moves — uses DECSC/DECRC save/restore each frame |
| XDG-compliant | Data in `~/.local/share/petcli/` — sprites cached locally after first fetch |

---

## Platform support

| Platform | Status | Notes |
|---|---|---|
| Linux | Full | Primary development target |
| macOS | Full | All features work — macOS is Unix |
| Windows | Supported | Uses `CONOUT$` for TTY, `taskkill`/`tasklist` for process management |

**Terminal requirements by platform:**

- **Linux / macOS** — any terminal (Kitty, Alacritty, WezTerm, Ghostty, iTerm2, etc.)
- **Windows** — Windows Terminal (recommended), or any VT-100-capable terminal; Kitty renderer requires Windows Terminal with GPU rendering enabled

---

## Install

**Requirements:** Rust stable toolchain, internet on first run.

```bash
git clone https://github.com/drayy/petcli
cd petcli
cargo install --path . --locked
```

On Windows, build in a terminal that supports ANSI escape sequences (Windows Terminal or VSCode terminal). The legacy `cmd.exe` prompt does not support the rendering.

---

## Usage

### Add a pet

Open the interactive TUI picker (recommended):

```bash
petcli add
```

Or add directly by name or Pokedex number:

```bash
petcli add pikachu
petcli add 25
petcli add eevee --shiny
```

### Picker controls

| Key | Action |
|---|---|
| `↑` / `↓` or `j` / `k` | Move selection |
| `/` | Search by name |
| `Tab` | Cycle generation filter (All → Gen 1 → 2 → 3 → 4) |
| `1` `2` `3` `4` `a` | Jump to a generation filter directly |
| `s` | Toggle shiny preview |
| `Enter` | Add selected Pokemon and exit |
| `q` / `Esc` | Quit without adding |

### All commands

```bash
petcli add [name|number] [--shiny]   # add a pet (no args = open picker)
petcli set <name|number> [--shiny]   # replace the whole party with one Pokemon
petcli status                        # show current party
petcli pokemon                       # list all 493 available Pokemon
petcli stop                          # kill the daemon and clear all pets
```

---

## How it works

`petcli` spawns a background daemon that re-opens the current terminal directly (`/dev/tty` on Linux/macOS, `CONOUT$` on Windows). Each frame:

1. Cursor position is saved with `\x1b7` (DECSC)
2. Previous sprite cells are erased
3. All pets are drawn at new positions
4. Cursor is restored with `\x1b8` (DECRC)

Your shell prompt and typing are completely unaffected. Pets bounce off all four terminal edges using a 2D velocity walker (think DVD screensaver). Terminal size is queried live via `TIOCGWINSZ` so resizing is handled correctly.

The daemon is managed via a PID file at `~/.local/share/petcli/daemon.pid` (Linux/macOS) or `%APPDATA%\petcli\daemon.pid` (Windows). `petcli stop` sends `SIGTERM` on Unix or runs `taskkill /F /PID` on Windows, then removes the file.

---

## Renderer detection

Auto-detected at startup:

| Terminal | Renderer |
|---|---|
| Kitty | Kitty graphics protocol (pixel-perfect, native 48×48) |
| WezTerm | Kitty graphics protocol |
| Ghostty | Kitty graphics protocol |
| Alacritty, tmux, everything else | Unicode half-blocks (`▀`) with ANSI 256-color |

Detection checks `$TERM == xterm-kitty` or `$KITTY_WINDOW_ID`. Override with `PETCLI_RENDERER=kitty` or `PETCLI_RENDERER=unicode`.

---

## Sprite sources

Sprites are fetched on first use and cached in `~/.local/share/petcli/sprites/`.

| Generation | Source |
|---|---|
| Gen 1–4 non-shiny | [jakobhoeg/vscode-pokemon](https://github.com/jakobhoeg/vscode-pokemon) — proper `idle_8fps.gif` + `walk_8fps.gif` |
| Gen 1–4 shiny | PokeAPI static shiny PNG (no animation available) |
| Gen 5 | PokeAPI Black/White animated sprites |
| Gen 6–9 | PokeAPI static PNG |

Walk frames play while the pet is moving; idle frames play when standing still (currently pets always walk).

---

## Known gaps

**Gen 5–9 walk animation** — The vscode-pokemon pack only covers Gen 1–4. PokeAPI has animated sprites for Gen 5 but only a single-direction loop, not a walk/idle split. Gen 6+ have no animation at all in any freely-licensed pack.

**Shiny Gen 1–4** — The vscode-pokemon pack ships non-shiny only. Shiny requests for Gen 1–4 fall back to a static PokeAPI PNG.

**Kitty multi-image erase** — The Kitty renderer issues `a=d,d=a` (delete all placements) before each redraw. If other Kitty images are on screen (e.g. from another tool), they would also be cleared.

**No true fork** — The daemon is launched via `Command::new(current_exe())` with a hidden `--daemon` flag rather than `fork(2)`. An extra `petcli` process will appear in `ps` (Linux/macOS) or Task Manager (Windows).

**Single terminal session** — The daemon is tied to the TTY of the terminal that launched it. Closing the window kills the daemon. There is no way to reattach.

**Party cap** — Hard-coded at 6 (mirrors the Pokemon game limit). Raising it is a one-line change in `src/state/pet.rs`.

---

## Architecture

```
src/
├── cli/
│   ├── mod.rs          clap command definitions (Set, Add, Status, Pokemon, Stop)
│   └── commands.rs     command handler implementations
├── animation/
│   └── walk.rs         2D bouncing walker — per-pet, independent velocity
├── renderer/
│   ├── kitty.rs        Kitty APC graphics protocol renderer
│   └── unicode.rs      half-block Unicode renderer (▀, ANSI 256-color)
├── sprite/
│   ├── fetch.rs        HTTP fetch + local disk cache (vscode-pokemon → PokeAPI)
│   ├── sheet.rs        SpriteSheet: idle + walk frame sequences
│   └── gif.rs          GIF decoder using `gif` + `gif-dispose`
├── state/
│   ├── persist.rs      JSON party save/load, XDG paths, PID file path
│   ├── pet.rs          Pet struct (species_id, shiny, alive)
│   └── pokedex.rs      static table of all 493 Pokemon names + gen_of() helper
├── tui/
│   ├── run.rs          daemon launcher + stop logic (spawn self, PID file)
│   ├── daemon.rs       main render loop (PetActor per pet, DECSC/DECRC, /dev/tty)
│   └── picker.rs       ratatui TUI picker (search, gen tabs, async sprite preview)
├── config.rs           config file (toml, ~/.config/petcli/)
└── error.rs            PetcliError (thiserror)
```

---

## References

- [jakobhoeg/vscode-pokemon](https://github.com/jakobhoeg/vscode-pokemon) — sprite source and the original inspiration; this project replicates that animation style in a raw terminal
- [Kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/) — pixel-level image rendering via APC escape sequences
- [PokeAPI/sprites](https://github.com/PokeAPI/sprites) — Gen 5 animated and static fallback sprites
- [ratatui](https://ratatui.rs) — TUI framework used for the picker
- [crossterm](https://github.com/crossterm-rs/crossterm) — cross-platform terminal control (cursor, color, raw mode)
- [gif-dispose](https://crates.io/crates/gif-dispose) — correct GIF frame compositing (handles disposal methods that plain `gif` ignores)

---

## License

MIT OR Apache-2.0
