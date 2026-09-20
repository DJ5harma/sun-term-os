# TDE: Project Vision, Architecture, and Roadmap

Status: Approved project direction  
Project: TDE — Terminal Desktop Environment  
Audience: Maintainers, contributors, and coding agents such as Cursor

## 1. Executive summary

TDE is intended to become a complete, lightweight desktop environment whose native interface is the terminal. It must feel like a coherent desktop, not a collection of unrelated terminal programs.

The long-term experience is:

> One lightweight desktop for your computer and all your computers.

A user should be able to work locally, open terminals, browse files, manage processes, launch applications, connect to remote machines, and eventually use remote GUI applications through one consistent environment.

The project is built incrementally. The first implementation is a cross-platform TUI shell. Remote machines, persistence, GUI rendering, and compositor functionality are future extensions and must not complicate the core before their requirements are real.

## 2. Product goal

TDE is a desktop environment replacement for users who want a fast, keyboard-friendly, terminal-native workspace without giving up the familiar mental model of a desktop.

It provides familiar desktop concepts:

- Desktop shell
- Numbered workspaces
- Application windows or surfaces
- Launcher and command palette
- Active-application/task area
- File manager
- Terminals
- Application launching
- System information and status
- Local and remote machines

The terminal is the native display and input environment. It is not merely a terminal emulator launched inside another desktop. TDE also must not rewrite every existing application; it should launch and manage existing CLI, TUI, and GUI applications through appropriate providers and frontends.

## 3. Product principles

### Familiar first

A new user should understand the desktop quickly. Use familiar patterns:

- Stable top bar for workspaces and machine identity
- Central desktop area for application windows
- Bottom area for active applications and status
- Discoverable launcher
- Consistent close, minimize, maximize, focus, and workspace behavior
- Clear loading, empty, and error states

TDE may have a distinctive visual identity, but basic interaction should not feel mysterious.

### Workspaces are general-purpose

A workspace is an instance of the desktop, not a fixed job role. Workspace 1 is not permanently a coding workspace and workspace 2 is not permanently a server workspace.

Users can launch any supported application in any workspace. Workspace names, profiles, persistence, and user customization are deferred until lifecycle requirements are understood. Early workspaces remain numbered: 1, 2, 3, and so on.

### Keyboard and mouse are equal input paths

Every important operation must have a keyboard path. Mouse support is required where it improves discoverability and direct manipulation.

Mouse hit testing must use the same geometry that rendering uses. Input code must not duplicate approximate coordinates or depend on visual assumptions.

### Fast, lightweight, and resilient

TDE should start quickly, remain responsive while services refresh, and work on small terminals. A failure in one capability must appear in that panel or operation without taking down the whole desktop.

### Remote support without local/remote forks

The UI and application core operate on capabilities, not on whether a machine is local or remote. SSH is an adapter behind the same interfaces, not a collection of SSH conditionals spread across the UI.

### Build only what the current phase needs

Do not build a compositor, GUI streaming protocol, remote agent, persistence system, plugin marketplace, or distributed state system before the product requirements justify it.

## 4. Target user experience

On startup, TDE presents:

1. A top bar showing TDE identity, machine, workspaces, and connection state.
2. A central area containing application windows.
3. A bottom bar showing active windows, current status, and useful shortcuts.
4. A launcher or command palette for discovering applications and actions.

The user can:

- Switch workspaces with keyboard or mouse.
- Open a terminal from the launcher or shortcut.
- Run commands in an embedded PTY-backed terminal.
- Interrupt a foreground command with Ctrl+C without closing the shell.
- Close a TDE terminal window with Ctrl+W.
- Browse files through the file manager.
- Inspect processes and system information.
- Launch supported applications from the same desktop.
- Later connect to a remote machine and use the same application concepts there.

The desktop must explain itself through labels, status messages, command palette entries, and consistent shortcuts. Hidden behavior is a usability bug.

## 5. Current implementation baseline

The current Rust binary establishes the initial shell architecture:

- Ratatui rendering with Crossterm input
- Tokio-based application runtime
- Numbered workspaces
- General-purpose application windows
- Launcher/command palette
- Local system information provider
- Local process provider
- Top bar, desktop area, and bottom bar
- Shared UI geometry for rendering and mouse hit testing
- Embedded terminal sessions using portable-pty
- ANSI terminal screen parsing using vt100
- Typed application state, actions, events, and effects
- Graceful terminal restoration through terminal startup/shutdown

The terminal is a TDE-owned application surface backed by a real PTY. TDE owns the surface and session lifecycle; the shell and programs inside it remain normal operating-system processes.

## 6. Architecture

The primary data flow is:

```
Keyboard / Mouse / Future Frontend Input
                    |
                    v
                 Event
                    |
                    v
                 Action
                    |
                    v
             Application Core
                    |
                    +--> Effect --> Capability Service
                    |                    |
                    |                    v
                    +<-- Result Event <--+
                    |
                    v
               Application State
                    |
                    v
                 Renderer
```

The renderer consumes immutable state. It must not call sysinfo, spawn a process, access the filesystem, open SSH, or make decisions based on local-versus-remote implementation details.

### Application core

The core owns:

- Application state
- Workspace and window relationships
- Focus and selection
- Launcher state
- Semantic action dispatch
- Effect scheduling
- Service-result application
- Quit and lifecycle decisions

State transitions should be small, explicit, and testable without starting a terminal or requiring a real machine.

### Events

Events represent things that happened:

- Keyboard or mouse input
- Timer tick
- System information result
- Process result
- Filesystem result
- Terminal output or exit
- Service error

Events carry domain-relevant data, not UI layout decisions.

### Actions

Actions represent user intent:

- Quit
- SwitchWorkspace(index)
- FocusNextWindow
- OpenApplication(kind)
- CloseWindow
- ToggleLauncher
- Refresh
- OpenDirectory(path)
- MoveFileSelection(offset)

Input translation converts physical keys and mouse events into actions. Reducers should not need to know which key produced an action.

### Effects and services

Effects request I/O, processes, or asynchronous services:

- Refresh capabilities
- Start or stop a terminal session
- Read a directory
- Launch an application
- Connect to an SSH machine

Results return as typed events and are applied to state.

## 7. Machine and capability architecture

Machines are first-class domain objects. A machine has:

- Stable MachineId
- Display name and metadata
- Machine kind: Local or future SshRemote
- A set of available capabilities

Use focused interfaces instead of one large Machine trait:

```
SystemInfoProvider
ProcessProvider
FilesystemProvider
TerminalProvider
ApplicationProvider
```

Each interface expresses one coherent capability. A local implementation may use operating-system APIs and mature crates. A future SSH implementation provides the same conceptual operations without changing UI rendering or workspace logic.

### Local adapter

Platform-specific behavior belongs inside local capability implementations. Do not scatter operating-system conditionals through reducers or widgets.

The local adapter owns details such as sysinfo snapshots, directory APIs, process launching, PTY creation, and platform-specific default application launching.

### Remote adapter

SSH should first be implemented as capability adapters:

- Machine connection and authentication
- Remote system information
- Remote process listing
- Remote filesystem listing
- Remote terminal sessions

An optional authenticated remote agent may be introduced only when SSH cannot provide a feature cleanly, such as persistent sessions, efficient streaming, or GUI transport.

## 8. Domain model

### Workspaces

Workspaces contain windows and focus state. They are independent desktop instances inside one TDE session and do not own fixed job types.

Early behavior:

- Workspaces are numbered.
- Count is fixed or simple in-memory state.
- No rename or persistence yet.
- Switching preserves each workspace’s open windows.

Future behavior may add names, dynamic creation, persistence, and remote placement only after the basic lifecycle is stable.

### Windows and applications

Applications are first-class objects. A window identifies an application surface, its state, and its owning workspace.

Early application kinds:

- Terminal
- File Manager
- Process Manager
- System Information

Later kinds may include text editor integration, service manager, remote machine browser, and GUI application surface.

Shared window lifecycle belongs in the domain. Application-specific state belongs in its application module.

### Terminal sessions

The terminal subsystem owns:

- PTY creation and resizing
- Shell process lifecycle
- Input forwarding
- Output reading
- ANSI parsing and screen state
- Exit detection
- Cleanup on close or application shutdown

Required semantics:

- Ctrl+C sends interrupt byte 0x03 to the foreground process.
- Ctrl+C does not close the shell or TDE window.
- Ctrl+W closes the TDE terminal window.
- An exited shell remains visible with final output and an exit status.
- Cleanup must not leave child processes or terminal modes behind.

## 9. File manager direction

The file manager is the next major desktop capability. It begins as a read-only browser to prove the capability boundary before mutations.

### First file manager scope

- Current directory display
- Directory entries
- File/directory type
- Size and modified time where available
- Keyboard selection movement
- Mouse selection
- Enter to open a directory
- Backspace or an action to move to the parent
- Hidden-file toggle as a discoverable action
- Loading, empty, permission-error, and missing-path states

Do not begin with delete, rename, copy, or move. Later mutations require explicit actions, destructive-operation confirmation, error reporting, local/remote correctness, and cancellation/partial-failure tests.

The file manager depends on FilesystemProvider. Widgets and reducers must not directly call std::fs.

## 10. Roadmap

Every phase must leave the application runnable and testable.

### Phase 0 — Foundation: complete

- Establish Rust binary and module boundaries.
- Create event/action/state/effect flow.
- Add local system and process capabilities.
- Build the initial Ratatui desktop shell.
- Add numbered general-purpose workspaces.

Acceptance: TDE starts and exits cleanly; system/process panels refresh; workspaces switch; launcher works; small terminals do not panic.

### Phase 1 — Terminal desktop foundation: substantially complete

- Add PTY-backed embedded terminal windows.
- Forward keyboard and terminal mouse input.
- Resize sessions with the rendered area.
- Track starting, running, exited, and failed states.
- Restore terminal state on shutdown.

Remaining hardening includes scrollback, selection behavior, and additional lifecycle tests.

### Phase 2 — File manager

- Add FilesystemProvider.
- Implement a read-only local filesystem adapter.
- Add a File Manager application window.
- Support navigation, selection, metadata, and errors.
- Keep filesystem work outside rendering code.

Acceptance: a new user can launch and navigate the file manager; permission and missing-path errors are visible and recoverable; keyboard and mouse use shared geometry; provider mapping and reducer transitions are tested.

### Phase 3 — Application launching and process UX

- Improve process manager interaction.
- Add safe local application discovery or explicit command launching.
- Open CLI/TUI programs through terminal sessions.
- Add platform-specific default application launching behind ApplicationProvider.
- Show launch failures inside the desktop.

No arbitrary remote command execution without an explicit authorization model.

### Phase 4 — Remote machines through SSH

- Add in-memory machine connection management.
- Implement SSH-backed system, process, filesystem, and terminal providers.
- Show local and remote machines through the same desktop concepts.
- Add connection status, authentication errors, host-key handling, and disconnect recovery.

The UI should not need local-versus-remote branches for normal capability use. Credentials and private keys must never be logged or casually persisted.

### Phase 5 — Session lifecycle and persistence

Only after workspace, machine, and application lifecycles are clear:

- Persist safe workspace/session metadata.
- Add workspace names and preferences.
- Restore selected sessions where safe.
- Define versioned configuration and migrations.

Do not add configuration storage merely to rename early numbered workspaces.

### Phase 6 — Persistent remote sessions

Evaluate SSH, tmux, mosh, or an authenticated TDE agent based on actual requirements. Any agent must be authenticated, encrypted, authorized, and disabled by default unless explicitly configured.

### Phase 7 — GUI application surfaces

GUI support is a future frontend/rendering problem. The target model is:

```
Application runs on local or remote machine
              |
              v
       TDE rendering client
```

Evaluate native windows, GPU rendering, and remote GUI transport only when requirements are known. Do not design a custom GUI protocol prematurely.

### Phase 8 — Full desktop/compositor integration

A Linux compositor or boot-to-TDE experience is a late-stage product track. It must not distort the cross-platform TUI architecture before the TUI DE is useful on its own.

## 11. Frontend evolution

The application core should support multiple frontends:

```
TUI frontend       ---> actions/state <--- future graphical frontend
                                      <--- future remote rendering client
```

A graphical frontend may require richer state and rendering primitives, but it must not duplicate local or SSH capability logic. The TUI must remain useful even if graphical support is never completed.

## 12. Errors and security

Use thiserror for typed domain/capability errors and anyhow at application boundaries. Never silently discard important failures. Show recoverable errors in the relevant panel or status area. Keep the desktop running when one provider or window fails.

Never:

- Expose an unauthenticated remote agent.
- Log passwords, tokens, or private keys.
- Disable SSH host verification silently.
- Store credentials casually in configuration files.
- Execute arbitrary remote commands without clear authorization.
- Treat a reachable remote machine as trusted automatically.

Remote features must use authenticated, encrypted transport and clearly expose connection failures.

## 13. Testing and definition of done

Run:

```
cargo fmt --check
cargo check
cargo clippy -- -D warnings
cargo test
```

Relevant tests cover:

- Reducer and state transitions
- Invalid workspace or selection handling
- Keyboard-to-action translation
- Mouse hit testing against shared geometry
- Provider-to-domain mapping
- Loading, empty, success, and error states
- Terminal input semantics and lifecycle
- Small-terminal layout calculations

A feature is done only when it works in the running application, keyboard and mouse paths work where applicable, errors are visible and recoverable, resizing is safe, existing functionality continues working, and all checks pass.

## 14. Rules for future coding agents

Read AGENTS.md and this document before changing architecture. Inspect the existing code first. Implement the smallest useful part of the current phase.

Always:

1. Keep UI rendering free of machine, filesystem, process, and SSH calls.
2. Add behavior through actions, effects, providers, and state transitions.
3. Reuse existing modules before adding abstractions.
4. Add tests for important state and provider behavior.
5. Keep the application usable on small terminals.
6. Run formatting, check, clippy, and tests.
7. Explain architectural decisions and deferred work in the handoff.

Do not, without explicit approval:

- Add SSH conditionals directly to widgets.
- Add persistence for convenience.
- Build a remote agent before SSH adapters are proven.
- Implement GUI streaming before the TUI model is stable.
- Add a compositor as part of a normal TUI feature.
- Add broad dependencies without a current requirement.
- Rewrite unrelated working modules.
- Perform destructive filesystem or remote operations without confirmation and tests.

Preferred change sequence:

```
Understand current code
        |
Define domain/action/state change
        |
Add provider/effect boundary if I/O is needed
        |
Implement reducer behavior
        |
Render immutable state
        |
Add keyboard and mouse paths
        |
Add tests
        |
Run all checks
```

When uncertain, prefer the design that keeps the core frontend-independent, local/remote implementations behind providers, the current phase small, failures visible, and future behavior possible.

## 15. North-star acceptance scenario

The project is moving in the right direction when a new user can:

1. Start TDE and immediately understand the desktop layout.
2. Open the launcher without reading source code.
3. Launch a terminal and run a command.
4. Interrupt that command without losing the shell.
5. Open the file manager and navigate to a directory.
6. Inspect running processes and system information.
7. Move between numbered workspaces while preserving applications.
8. Understand errors from the interface and recover.
9. Later connect to another machine and use the same concepts remotely.

The standard is not merely a better terminal. It is a coherent, lightweight desktop environment for one computer and all of a user’s computers.

