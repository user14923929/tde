# TDE — Terminal Desktop Environment

A minimal pseudo-DE that lives entirely in your terminal.
Written in Rust using [ratatui](https://ratatui.rs).

## Features

- **Tiling layout** — split panes horizontally or vertically
- **Launcher** — fuzzy-search app launcher (Space)
- **Statusbar** — live CPU, RAM, clock
- **Single TOML config** — `~/.config/tde/config.toml`

## Keybinds

| Key          | Action               |
|--------------|----------------------|
| `Space`      | Open launcher        |
| `Tab`        | Focus next pane      |
| `BackTab`    | Focus previous pane  |
| `H`          | Split horizontal     |
| `V`          | Split vertical       |
| `X`          | Close focused pane   |
| `Q` / `Ctrl+C` | Quit               |

## Install

```bash
git clone https://gitlab.com/user14923929/tde.git
cd tde
cargo build --release
./target/release/tde
```

## Config

On first run, a default config is written to `~/.config/tde/config.toml`:

```toml
[theme]
name = "default"
accent = "cyan"

[keybinds]
quit = "q"
open_launcher = "space"
split_horizontal = "h"
split_vertical = "v"
close_pane = "x"

[statusbar]
show_clock = true
show_cpu = true
show_mem = true
position = "Bottom"
```

## Roadmap

- [ ] Embedded PTY (actual shell inside pane)
- [ ] Plugin system (Lua scripts)
- [ ] Notification daemon
- [ ] Workspace switcher
- [ ] Mouse support

## License

MIT