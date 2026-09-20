# sun-term-os

A lightweight, terminal-native desktop environment written in Rust.

## Requirements

- Rust stable (2024 edition)
- A POSIX shell for embedded terminals (`$SHELL` or `sh`)

## Build and run

```bash
cargo run
```

Configuration lives in a single file: `$XDG_CONFIG_HOME/sun-term-os/config.toml` (or `~/.config/sun-term-os/config.toml`). Edit it in the **Settings** app (command palette → Settings) and press **s** or choose **Save settings to disk** — no CLI flags.

`refresh_interval_secs` is clamped to 1–300. `workspace_count` is clamped to 1–9 (switch with F1–Fn); change workspace count in the config file if needed.

Release build:

```bash
cargo build --release
./target/release/sun-term-os
```

## Development

```bash
cargo fmt
cargo check
cargo clippy -- -D warnings
cargo test
```

## Shortcuts

| Action | Keys |
|--------|------|
| Quit | `Ctrl+C` or `q` (when not in a terminal) |
| Command palette | `Alt+P` or click **⊞ Palette** (type to filter, ↑↓, Enter; `!cmd` runs in a new terminal) |
| Settings | Palette → **Settings** · pick a **color theme** preset · `s` to save |
| New terminal | `t` |
| File manager | `f` |
| Process manager | `p` |
| System info | `s` |
| Machines (SSH profiles) | `m` |
| Application launcher | `l` |
| Text viewer | `v` · `:` open path · file manager `e` on a file |
| Service manager | Palette → **Service Manager** |
| Refresh system/process data | `r` |
| Workspaces | `F1`–`F3` or click workspace tabs |
| Focus window by slot | `Ctrl+G` then `1`–`9` (matches bottom bar order) |
| Close / minimize / maximize window | `Ctrl+W` / `Ctrl+M` / `Ctrl+F` |
| File manager: open in terminal | `o` |
| File manager: trash/delete (confirm) | `d` or `Delete` (uses system trash when available) |
| File manager: rename | `Shift+R` |
| File manager: new file / folder | `a` / `Shift+A` |
| Process manager: filter | `/` then type; `Esc` or `Enter` to finish |
| Process manager: SIGTERM / SIGKILL | `x` / `Shift+X` on selected row |
| Process manager: sort | `1` CPU · `2` memory · `3` name · `4` PID |
| File manager: go to path | `:` |
| File manager: open with OS handler | `Shift+O` |
| Text viewer: open path | `:` |
| File manager: open in text viewer | `e` (files) |
| Machines: connect / disconnect | `c` / `d` · add host `a` |
| Launcher: filter / launch / pin favorite | `/` · Enter · Shift+F (saved to config) |
| Services: start / stop / restart | `s` / `x` / `r` |
| Terminal: scrollback | `Alt+PgUp` / `Alt+PgDn` · `Alt+End` follow live |
| Input debug overlay | `Ctrl+Alt+D` |

Embedded terminals receive all other keys. Host terminals often steal `Ctrl+digit`; the `Ctrl+G` chord is intentional.

An empty workspace shows a **home screen** (app tiles): click a tile, press **Enter**, or use letter shortcuts (`t`, `f`, …). Pin order is configurable under `[home] pinned` in `config.toml`.

## Architecture

Input is normalized and routed by focus. Actions update `AppState` through a reducer; side effects (PTY, filesystem reads, signals) run in the app loop. Machine capabilities live behind async provider traits so remote machines can share the same UI later.

See [AGENTS.md](AGENTS.md) for project goals and conventions.
