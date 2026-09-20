# AGENTS.md

## Project: sun-term-os

TDE is a lightweight, cross-platform, terminal-native desktop environment.

The long-term vision is to combine:

- A full TUI desktop environment
- Local CLI/TUI/GUI application management
- SSH-based remote machine management
- Persistent remote sessions
- Remote GUI application support
- Eventually, a full Linux desktop environment / compositor

The terminal is the **native interface**, not merely a terminal app running inside another desktop.

## Current Goal

Build TDE incrementally.

For the current stage, prioritize the **TUI desktop experience**. Do not implement future GUI streaming, compositor, or remote-desktop features unless the current task explicitly requires them.

The immediate goal is a polished terminal desktop with:

- Desktop shell
- Workspaces
- Command palette
- Embedded terminal
- File manager
- Process manager
- System information
- Eventually SSH/remote machines

## Technology

Primary language:

- Rust (stable)

Initial stack:

- Ratatui — TUI
- Crossterm — terminal backend
- Tokio — async runtime
- Serde — serialization
- TOML — configuration
- anyhow — application-level errors
- thiserror — library/domain errors
- tracing + tracing-subscriber — logging
- clap — CLI
- sysinfo — system/process information
- portable-pty — embedded terminal
- notify — filesystem watching
- russh — SSH

Later, when GUI support is actually being implemented:

- winit — native windows/input
- wgpu — GPU rendering

## Architecture Principles

### 1. Keep the core independent of the UI

Prefer:

    User input
      -> Event
      -> Action
      -> Core/service
      -> State update
      -> UI render

Do not put filesystem, process, SSH, or other business logic directly inside rendering code.

### 2. Treat machines as first-class objects

The architecture should eventually support:

    LocalMachine
    RemoteMachine

Both should expose the same conceptual capabilities:

- Filesystem
- Terminal
- Processes
- Services
- Applications
- System information

The UI should not need to know whether it is operating on localhost or a remote machine.

### 3. Design for remote support, but do not overbuild it

Avoid hard-coding assumptions that everything is local.

However, do not create a complex distributed architecture before remote functionality is actually needed.

### 4. Applications are first-class

Eventually TDE should understand:

- TUI applications
- CLI applications
- GUI applications

Do not try to rewrite or emulate applications that already exist. TDE should launch and manage them.

### 5. Keep platform-specific code isolated

Cross-platform logic belongs in common modules.

OS-specific behavior should live behind clear platform abstractions for:

- Linux
- macOS
- Windows

Do not scatter `cfg(target_os = ...)` throughout unrelated UI code.

## Vibecoding Rules

### Prefer simple code

When several designs work, choose the simplest one that keeps the architecture extensible.

Do not introduce:

- unnecessary abstractions
- unnecessary traits
- premature microservices
- excessive crates
- complex dependency injection
- elaborate event systems without a real need

### Do not over-engineer future phases

Do not build:

- a Wayland compositor before needed
- a custom remote desktop protocol before existing protocols are evaluated
- a GUI renderer before GUI support is being implemented
- a plugin marketplace before the plugin model exists
- a distributed state system before remote state is needed

Build the smallest useful version first.

### Keep dependencies justified

Before adding a crate, ask:

1. Is it solving a real current problem?
2. Is it mature enough for the project?
3. Could the existing stack handle it simply?

Avoid dependency sprawl.

### Make changes incrementally

Prefer small, working changes over huge rewrites.

After meaningful changes, run:

    cargo fmt
    cargo check
    cargo clippy
    cargo test

Fix regressions before continuing.

## Code Quality

Write idiomatic, readable Rust.

Prefer:

- Small functions
- Clear names
- Explicit state transitions
- Strong types
- Error handling
- Minimal mutable global state
- Tests around important logic

Avoid:

- `unwrap()` in production paths unless failure is genuinely impossible
- giant files
- giant functions
- duplicated business logic
- hidden side effects
- UI code that directly shells out for everything

Comments should explain **why**, not simply restate what the code does.

## UI Principles

TDE should feel like a desktop, not a collection of unrelated terminal programs.

Prioritize:

- Fast keyboard navigation
- Mouse support where useful
- Consistent navigation
- Consistent shortcuts
- Clear focus state
- Responsive layouts
- Good empty/loading/error states
- Search and command discovery
- Minimal visual clutter

The UI must degrade reasonably on small terminal sizes.

Never assume a large terminal.

## Project Structure

Start simple.

Prefer a single binary crate with logical modules until there is a real reason to split into multiple crates.

A reasonable early structure:

    src/
    ├── main.rs
    ├── app.rs
    ├── state.rs
    ├── events.rs
    ├── actions.rs
    ├── ui/
    │   ├── mod.rs
    │   ├── layout.rs
    │   ├── widgets.rs
    │   └── theme.rs
    ├── terminal/
    │   └── mod.rs
    ├── machine/
    │   └── mod.rs
    └── config/
        └── mod.rs

Refactor into crates only when boundaries become useful.

## Development Workflow

When implementing a task:

1. Read the existing code before changing it.
2. Identify the smallest implementation that satisfies the task.
3. Reuse existing abstractions.
4. Implement the change.
5. Run formatting, checking, linting, and tests.
6. Fix any regressions.
7. Summarize what changed and any important architectural decisions.

Do not silently change project architecture just because another architecture seems cleaner.

## Git

Make focused commits.

Good examples:

    feat: add workspace switching
    feat: add embedded terminal
    feat: add process manager
    fix: preserve terminal size on resize
    refactor: isolate machine operations

Avoid mixing unrelated features in one commit.

## Security

TDE will eventually control remote machines.

Treat security as important from the beginning.

Never:

- expose an unauthenticated remote agent
- execute arbitrary remote commands without authorization
- log passwords or private keys
- store credentials casually
- bypass host verification without an explicit reason

Remote capabilities should use authentication and encryption.

## Definition of Done

A feature is not done merely because it compiles.

For a feature to be considered done:

- It works in the running TDE application.
- Keyboard interaction works.
- Errors are handled sensibly.
- Terminal resizing does not obviously break it.
- Existing features continue working.
- `cargo fmt` passes.
- `cargo check` passes.
- `cargo clippy` passes without introducing avoidable warnings.
- Relevant tests pass.

## Important North Star

Build toward this experience:

    One lightweight desktop for your computer and all your computers.

A user should eventually be able to:

- Work locally
- Open terminals
- Manage files
- Manage processes and services
- Launch applications
- Connect to remote machines
- Use remote files and processes
- Run remote GUI applications
- Move between local and remote workspaces

All through one coherent TDE experience.

Do not lose this goal while implementing individual features.
