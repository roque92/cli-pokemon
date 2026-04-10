# petcli

Animated Pokemon pets living in your terminal.

> The gap: `krabby` shows static Unicode art. `vscode-pokemon` shows animated pixel sprites — but only inside VS Code. **petcli** brings that to the CLI.

---

## Features

- **Animated pixel sprites** via [Kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/) — works in Kitty, WezTerm, Ghostty
- **Unicode half-block art fallback** for Alacritty and any other terminal
- **Full tamagotchi sim** — hunger, happiness, energy decay in real time
- **Party of 6** — catch eggs, hatch them, evolve your Pokemon
- **Level-up & evolution** — level-based and happiness-based chains (full Gen 1)
- **Items & shop** — buy food, toys, medicine with coins earned from interactions
- **Shiny variants** — 1/8192 chance on catch (configurable)
- **Compact mode** — minimal pane layout for tmux/zellij ricers
- **Config file** — tune decay rate, shiny odds, sprite size

---

## Install

```bash
cargo install petcli
```

Or build from source:

```bash
git clone https://github.com/drayy/petcli
cd petcli
cargo build --release
./target/release/petcli
```

---

## Usage

```bash
petcli              # Launch TUI (animated pet in full layout)
petcli --compact    # Compact mode: minimal pane for tmux/zellij

petcli catch        # Catch a random egg
petcli status       # Show party stats
petcli feed         # Feed active Pokemon
petcli pet          # Pet active Pokemon
petcli inventory    # Show wallet and items
petcli config       # Print config file location
```

### TUI Keybinds

| Key | Action |
|-----|--------|
| `f` | Feed active Pokemon |
| `p` | Pet active Pokemon |
| `u` | Use best item from inventory |
| `s` | Open shop |
| `Tab` | Cycle party |
| `q` / `Esc` | Quit |

**In shop:**

| Key | Action |
|-----|--------|
| `↑` / `k` | Navigate up |
| `↓` / `j` | Navigate down |
| `Enter` | Buy (confirm on second press) |
| `n` / `Esc` | Cancel / close |

---

## Terminal Support

| Terminal | Renderer |
|----------|----------|
| Kitty | Pixel sprites (Kitty graphics protocol) |
| WezTerm | Pixel sprites (Kitty graphics protocol) |
| Ghostty | Pixel sprites (Kitty graphics protocol) |
| Alacritty | Unicode half-block art (`▀▄` + ANSI 256-color) |
| Everything else | Unicode half-block art |

---

## Configuration

Config lives at `~/.config/petcli/config.toml` (created automatically on first run of `petcli config`):

```toml
[display]
compact = false        # default to compact mode
sprite_size = 96       # pixel size for Kitty renderer
show_shiny_indicator = true

[game]
shiny_rate = 8192      # 1-in-N chance of shiny on catch
decay_rate = 1.0       # stat decay speed multiplier
coins_per_feed = 2
coins_per_pet = 5
```

---

## How It Works

- **Sprites** are fetched on demand from the [PokeAPI sprites repo](https://github.com/PokeAPI/sprites) (Gen 5 animated GIFs) and cached to `~/.local/share/petcli/sprites/`. Nothing is bundled in the binary.
- **Kitty rendering** uploads all GIF frames as a multi-frame Kitty animation using APC escape sequences. Kitty handles frame timing internally. The sprite is re-placed when the pet moves.
- **Unicode rendering** pre-converts every frame to `▀` half-block characters at sprite load time. Each channel is mapped to the nearest xterm-256 color cube entry.
- **State** persists to `~/.local/share/petcli/state.json`. Stat decay is calculated from the `last_updated` timestamp on each load, so your pet degrades even while petcli is closed.

---

## Project Structure

```
src/
├── animation/      # State machine, frame ticker, walk AI
├── cli/            # Clap subcommands
├── config.rs       # Config file (toml)
├── error.rs        # PetcliError (thiserror)
├── renderer/       # Kitty + Unicode renderers, auto-detection
├── sim/            # Egg hatching, evolution, interactions
├── sprite/         # GIF fetch/cache/decode (gif-dispose for correct compositing)
├── state/          # Pet, Party, Inventory, decay, persistence
└── tui/            # ratatui layout, widgets, event loop
```

---

## License

MIT OR Apache-2.0
