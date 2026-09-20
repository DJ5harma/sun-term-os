# TDE

A lightweight, terminal-native desktop environment written in Rust.

## Requirements

- Rust stable (2024 edition)
- A POSIX shell for embedded terminals (`$SHELL` or `sh`)

## Build and run

```bash
cargo run
```

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
| App launcher | `Alt+P` or click **⊞ Apps** |
| New terminal | `t` |
| File manager | `f` |
| Process manager | `p` |
| System info | `s` |
| Refresh system/process data | `r` |
| Workspaces | `F1`–`F3` or click workspace tabs |
| Focus window by slot | `Ctrl+G` then `1`–`9` (matches bottom bar order) |
| Close / minimize / maximize window | `Ctrl+W` / `Ctrl+M` / `Ctrl+F` |
| File manager: open in terminal | `o` |
| Process manager: filter | `/` then type; `Esc` or `Enter` to finish |
| Process manager: SIGTERM | `x` on selected row |
| Input debug overlay | `Ctrl+Alt+D` |

Embedded terminals receive all other keys. Host terminals often steal `Ctrl+digit`; the `Ctrl+G` chord is intentional.

## Architecture

Input is normalized and routed by focus. Actions update `AppState` through a reducer; side effects (PTY, filesystem reads, signals) run in the app loop. Machine capabilities live behind async provider traits so remote machines can share the same UI later.

See [AGENTS.md](AGENTS.md) for project goals and conventions.
