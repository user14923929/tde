# TDE — Terminal Desktop Environment

A keyboard-driven pseudo-DE that lives entirely in your terminal.
Written in Rust using [ratatui](https://ratatui.rs). No mouse required.

## Features

- **Tiling layout** — split panes horizontally (`H`) or vertically (`V`)
- **Workspace switcher** — up to 9 independent workspaces (`1`–`9`, `[`/`]`)
- **Launcher** — fuzzy-search app launcher (`Space`)
- **Embedded PTY** — real shell inside each pane (v0.1.1)
- **Lua plugin system** — scripts in `~/.config/tde/plugins/*.lua`
- **Notification daemon** — toast overlays, usable from plugins via `tde.notify()`
- **Statusbar** — live CPU, RAM, clock
- **Single TOML config** — `~/.config/tde/config.toml`
- **100% keyboard-driven** — no mouse needed

## Keybinds

| Key            | Action                    |
|----------------|---------------------------|
| `Space`        | Open launcher             |
| `Tab`          | Focus next pane           |
| `BackTab`      | Focus previous pane       |
| `H`            | Split pane horizontally   |
| `V`            | Split pane vertically     |
| `X`            | Close focused pane        |
| `1`–`9`        | Switch to workspace N     |
| `[` / `]`      | Previous / next workspace |
| `Q` / `Ctrl+C` | Quit                      |

## Install

```bash
git clone https://github.com/user14923929/tde.git
cd tde
cargo build --release
./target/release/tde
```

## Lua Plugins

Place `.lua` files in `~/.config/tde/plugins/`. Available API:

```lua
-- Called on startup
function on_startup()
    tde.notify("Hello from plugin!")
end

-- tde.version  →  "0.1.1"
-- tde.notify(msg)  →  shows a toast notification
```

## Config

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

- [x] Tiling layout
- [x] Launcher
- [x] Statusbar
- [x] App launch (suspend/restore TUI)
- [x] Embedded PTY
- [x] Lua plugin system
- [x] Notification daemon
- [x] Workspace switcher
- [ ] Mouse-free file manager integration
- [ ] Plugin hot-reload

## License

MIT