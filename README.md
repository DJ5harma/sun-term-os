# TDE

A lightweight, terminal-native desktop environment written in Rust.

## Requirements

- Rust stable (2024 edition)
- A POSIX shell for embedded terminals (`$SHELL` or `sh`)

## Build and run

```bash
cargo run
```

Optional config path:

```bash
cargo run -- --config /path/to/config.toml
```

Default config file: `$XDG_CONFIG_HOME/tde/config.toml` or `~/.config/tde/config.toml`.

Example:

```toml
refresh_interval_secs = 5
workspace_count = 4
```

`refresh_interval_secs` is clamped to 1–300. `workspace_count` is clamped to 1–9 (switch with F1–Fn).

Release build:

```bash
cargo build --release
./target/release/tde
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
| New terminal | `t` |
| File manager | `f` |
| Process manager | `p` |
| System info | `s` |
| Refresh system/process data | `r` |
| Workspaces | `F1`–`F3` or click workspace tabs |
| Focus window by slot | `Ctrl+G` then `1`–`9` (matches bottom bar order) |
| Close / minimize / maximize window | `Ctrl+W` / `Ctrl+M` / `Ctrl+F` |
| File manager: open in terminal | `o` |
| File manager: trash/delete (confirm) | `d` or `Delete` (uses system trash when available) |
| File manager: rename | `Shift+R` |
| File manager: new file / folder | `a` / `Shift+A` |
| Process manager: filter | `/` then type; `Esc` or `Enter` to finish |
| Process manager: SIGTERM | `x` on selected row |
| Input debug overlay | `Ctrl+Alt+D` |

Embedded terminals receive all other keys. Host terminals often steal `Ctrl+digit`; the `Ctrl+G` chord is intentional.

## Architecture

Input is normalized and routed by focus. Actions update `AppState` through a reducer; side effects (PTY, filesystem reads, signals) run in the app loop. Machine capabilities live behind async provider traits so remote machines can share the same UI later.

See [AGENTS.md](AGENTS.md) for project goals and conventions.
