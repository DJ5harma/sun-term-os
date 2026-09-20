# TDE architecture

## Data flow

Input → `Event` → `Action` → `reduce` → `Effect` → `EffectExecutor` → `Action::Async` → render.

## Built-in apps (`src/apps/`)

Each built-in owns a module with:

- `APP` — `BuiltInApp` registry entry (metadata, lifecycle, render, keys, scroll, palette)
- `on_open` / `on_close` — lifecycle + initial `Effect`s
- `dispatch_key` — focused-window keyboard handling (`AppKeyResult`)
- `run_effect` — async side effects for that app’s `Effect` variants
- Optional `palette_extras` (file manager today)

[`apps/mod.rs`](../src/apps/mod.rs) is the **dispatch table**: lookup by `ApplicationKind`, palette helpers, `run_effect` routing.

The desktop shell ([`ui/windows.rs`](../src/ui/windows.rs)) only draws window chrome and calls `apps::render`.

Keyboard: [`input/router.rs`](../src/input/router.rs) handles globals and the launcher, then `apps::dispatch_key`. Chrome / empty desktop uses `apps/shell_keys.rs`.

Effects are grouped per app in [`app/effects.rs`](../src/app/effects.rs) (`TerminalEffect`, `FileManagerEffect`, `ProcessEffect`); execution lives in each app module.

### Adding an app

1. Add `ApplicationKind` in [`domain/application.rs`](../src/domain/application.rs)
2. New `src/apps/<name>.rs` with `APP`, lifecycle, `render`, `dispatch_key`, and `run_effect` if needed
3. Register `APP` in `BUILT_INS` in [`apps/mod.rs`](../src/apps/mod.rs)
4. Wire reducer/actions only for app-specific `Action` variants (if any)

## Other modules

| Area | Role |
|------|------|
| [`app/reducer/`](../src/app/reducer/) | State transitions by action kind |
| [`app/state.rs`](../src/app/state.rs) | Shell + per-window app state |
| [`machine/`](../src/machine/) | Capability providers |
| [`app/runtime/`](../src/app/runtime/) | Main loop and effects |

System info and process list data are **per window** (`system_info_views`, `ProcessManagerState::listing`), with global copies kept for refresh and the bottom bar.
