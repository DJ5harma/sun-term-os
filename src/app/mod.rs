mod effects;
pub mod file_manager;
pub mod palette;
pub mod process_manager;
mod reducer;
pub mod state;

pub use crate::domain::{ApplicationKind, Window, WindowState};
pub use state::{AppState, FileManagerFocus, Loadable, SortColumn, TerminalStatus};

use std::{sync::Arc, thread, time::Duration};

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent};
use ratatui::DefaultTerminal;
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tokio::time::{self, MissedTickBehavior};

use crate::{
    actions::Action,
    config::Config,
    events::Event,
    input,
    machine::{FilesystemProvider, ProcessProvider, SystemInfoProvider},
    terminal::{TerminalEvent, TerminalManager},
    ui,
};

use effects::Effect;

pub struct App {
    pub state: AppState,
    refresh_interval_secs: u64,
    geometry: ui::geometry::UiGeometry,
    mouse_click: input::DoubleClickState,
    interactions: ui::interaction::InteractionMap,
    terminal_manager: TerminalManager,
    system_provider: Arc<dyn SystemInfoProvider>,
    process_provider: Arc<dyn ProcessProvider>,
    filesystem_provider: Arc<dyn FilesystemProvider>,
}

impl App {
    pub fn new(
        config: Config,
        system_provider: Arc<dyn SystemInfoProvider>,
        process_provider: Arc<dyn ProcessProvider>,
        filesystem_provider: Arc<dyn FilesystemProvider>,
    ) -> Self {
        let config = config.normalized();
        Self {
            state: AppState::new(config.workspace_count),
            refresh_interval_secs: config.refresh_interval_secs,
            geometry: ui::geometry::UiGeometry::default(),
            mouse_click: input::DoubleClickState::default(),
            interactions: ui::interaction::InteractionMap::default(),
            terminal_manager: TerminalManager::default(),
            system_provider,
            process_provider,
            filesystem_provider,
        }
    }

    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let mut ticker = time::interval(Duration::from_secs(self.refresh_interval_secs));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        let mut input_events = spawn_input_reader();
        self.dispatch(Action::Refresh).await;

        while !self.state.should_quit {
            let size = terminal.size()?;
            self.geometry = ui::geometry::calculate(
                ratatui::layout::Rect::new(0, 0, size.width, size.height),
                &self.state,
            );
            self.sync_file_manager_visible_rows();
            self.sync_process_manager_visible_rows();
            self.sync_palette_selection();
            self.resize_focused_terminal();
            self.interactions.clear();
            terminal.draw(|frame| {
                ui::render(frame, &self.state, &self.geometry, &mut self.interactions)
            })?;

            tokio::select! {
                _ = ticker.tick() => self.handle_event(Event::Tick).await,
                terminal_event = self.terminal_manager.events.recv() => {
                    if let Some(event) = terminal_event {
                        self.handle_terminal_event(event);
                    }
                }
                event = input_events.recv() => {
                    if let Some(event) = event {
                        self.handle_event(event).await;
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => {
                let terminal_window = self
                    .state
                    .focused_window()
                    .filter(|window| window.application == ApplicationKind::Terminal)
                    .map(|window| window.id);
                let focus = if self.state.launcher_open {
                    input::FocusContext::Launcher
                } else if let Some(window) = self.state.focused_window() {
                    input::FocusContext::Window(window.application)
                } else {
                    input::FocusContext::Chrome
                };
                let process_filter_active = self
                    .state
                    .focused_window()
                    .filter(|window| window.application == ApplicationKind::Processes)
                    .and_then(|window| self.state.process_manager(window.id))
                    .is_some_and(|manager| manager.filter_active);
                let file_manager_dialog = {
                    let manager = self
                        .state
                        .focused_window()
                        .filter(|window| window.application == ApplicationKind::FileManager)
                        .and_then(|window| self.state.file_manager(window.id))
                        .or_else(|| {
                            crate::app::file_manager::target_file_manager_window(&self.state)
                                .and_then(|id| self.state.file_manager(id))
                        });
                    manager
                        .map(|manager| match &manager.dialog {
                            state::FileManagerDialog::None => input::FileManagerDialogMode::None,
                            state::FileManagerDialog::DeleteConfirm { .. } => {
                                input::FileManagerDialogMode::DeleteConfirm
                            }
                            state::FileManagerDialog::Rename { .. } => {
                                input::FileManagerDialogMode::Rename
                            }
                            state::FileManagerDialog::Create { .. } => {
                                input::FileManagerDialogMode::Create
                            }
                        })
                        .unwrap_or(input::FileManagerDialogMode::None)
                };
                let context = input::KeyInputContext {
                    focus,
                    window_pick_mode: self.state.window_pick_mode,
                    process_filter_active,
                    file_manager_dialog,
                };
                let dispatch = input::handle_key(key, &context);
                if self.state.input_debug {
                    self.state.input_debug_line = format_input_debug(key, &dispatch);
                }
                match dispatch {
                    input::KeyDispatch::Action(action) => self.dispatch(action).await,
                    input::KeyDispatch::Terminal(bytes) => {
                        if let Some(window_id) = terminal_window {
                            let _ = self.terminal_manager.write_input(window_id, &bytes);
                        }
                    }
                    input::KeyDispatch::Consumed => {}
                }
            }
            Event::Mouse(mouse) => {
                let actions = input::actions_for_mouse(
                    mouse,
                    &self.state,
                    &self.geometry,
                    &self.interactions,
                    &mut self.mouse_click,
                );
                for action in actions {
                    self.dispatch(action).await;
                }
            }
            Event::SystemInfoLoaded(result) => {
                self.state.apply_event(Event::SystemInfoLoaded(result))
            }
            Event::ProcessesLoaded(result) => {
                self.state.apply_event(Event::ProcessesLoaded(result))
            }
            Event::DirectoryLoaded(window_id, result) => self
                .state
                .apply_event(Event::DirectoryLoaded(window_id, result)),
            Event::Tick => self.dispatch(Action::Refresh).await,
        }
    }

    async fn dispatch(&mut self, action: Action) {
        let effects = reducer::reduce(&mut self.state, action);
        self.run_effects(effects).await;
    }

    async fn run_effects(&mut self, effects: Vec<Effect>) {
        for effect in effects {
            match effect {
                Effect::RefreshCapabilities => self.refresh_capabilities().await,
                Effect::StartTerminal(window_id) => {
                    if let Err(error) = self.terminal_manager.open(window_id, 80, 24) {
                        self.state.set_terminal_status(
                            window_id,
                            TerminalStatus::Failed(error.to_string()),
                        );
                        self.state.status = format!("Terminal failed to start: {error}");
                    } else {
                        self.state
                            .set_terminal_status(window_id, TerminalStatus::Running);
                    }
                }
                Effect::StopTerminal(window_id) => {
                    self.terminal_manager.close(window_id);
                    self.state.remove_terminal_content(window_id);
                    self.state.remove_terminal_status(window_id);
                }
                Effect::ReadDirectory(window_id, path) => {
                    let show_hidden = self
                        .state
                        .file_manager(window_id)
                        .map(|manager| manager.show_hidden)
                        .unwrap_or(false);
                    let result = self
                        .filesystem_provider
                        .list_directory(&path, show_hidden)
                        .await
                        .map_err(|error| error.to_string());
                    self.state
                        .apply_event(Event::DirectoryLoaded(window_id, result));
                }
                Effect::WriteTerminal(window_id, bytes) => {
                    let _ = self.terminal_manager.write_input(window_id, &bytes);
                }
                Effect::KillProcess(pid) => {
                    let result = self.process_provider.kill_process(pid).await;
                    match result {
                        Ok(()) => self.state.status = format!("Sent SIGTERM to PID {pid}"),
                        Err(error) => self.state.status = format!("Kill failed: {error}"),
                    }
                    self.refresh_capabilities().await;
                }
                Effect::DeletePath(window_id, path) => {
                    let directory = self
                        .state
                        .file_manager(window_id)
                        .map(|manager| manager.current_path.clone());
                    let result = self.filesystem_provider.remove_path(&path).await;
                    if let Some(manager) = self.state.file_manager_mut(window_id) {
                        manager.dialog = state::FileManagerDialog::None;
                    }
                    match result {
                        Ok(crate::machine::RemoveOutcome::MovedToTrash) => {
                            self.state.status = format!("Moved to trash: {}", path.display());
                            if let Some(directory) = directory {
                                self.schedule_read_directory(window_id, directory).await;
                            }
                        }
                        Ok(crate::machine::RemoveOutcome::DeletedPermanently) => {
                            self.state.status = format!("Deleted permanently: {}", path.display());
                            if let Some(directory) = directory {
                                self.schedule_read_directory(window_id, directory).await;
                            }
                        }
                        Err(error) => {
                            self.state.status = format!("Remove failed: {error}");
                        }
                    }
                }
                Effect::CreateEntry(window_id, path, kind) => {
                    let directory = self
                        .state
                        .file_manager(window_id)
                        .map(|manager| manager.current_path.clone());
                    let result = match kind {
                        crate::app::file_manager::CreateKind::File => {
                            self.filesystem_provider.create_file(&path).await
                        }
                        crate::app::file_manager::CreateKind::Directory => {
                            self.filesystem_provider.create_directory(&path).await
                        }
                    };
                    if let Some(manager) = self.state.file_manager_mut(window_id) {
                        manager.dialog = state::FileManagerDialog::None;
                    }
                    match result {
                        Ok(()) => {
                            self.state.status = format!("Created {}", path.display());
                            if let Some(directory) = directory {
                                self.schedule_read_directory(window_id, directory).await;
                            }
                        }
                        Err(error) => self.state.status = format!("Create failed: {error}"),
                    }
                }
                Effect::RenamePath(window_id, from, to) => {
                    let directory = self
                        .state
                        .file_manager(window_id)
                        .map(|manager| manager.current_path.clone());
                    let result = self.filesystem_provider.rename_path(&from, &to).await;
                    if let Some(manager) = self.state.file_manager_mut(window_id) {
                        manager.dialog = state::FileManagerDialog::None;
                    }
                    match result {
                        Ok(()) => {
                            self.state.status = format!(
                                "Renamed {} → {}",
                                from.file_name()
                                    .map(|name| name.to_string_lossy())
                                    .unwrap_or_default(),
                                to.file_name()
                                    .map(|name| name.to_string_lossy())
                                    .unwrap_or_default()
                            );
                            if let Some(directory) = directory {
                                self.schedule_read_directory(window_id, directory).await;
                            }
                        }
                        Err(error) => {
                            self.state.status = format!("Rename failed: {error}");
                        }
                    }
                }
            }
        }
    }

    async fn schedule_read_directory(&mut self, window_id: u64, path: std::path::PathBuf) {
        if let Some(manager) = self.state.file_manager_mut(window_id) {
            manager.listing = Loadable::Loading;
        }
        let show_hidden = self
            .state
            .file_manager(window_id)
            .map(|manager| manager.show_hidden)
            .unwrap_or(false);
        let result = self
            .filesystem_provider
            .list_directory(&path, show_hidden)
            .await
            .map_err(|error| error.to_string());
        self.state
            .apply_event(Event::DirectoryLoaded(window_id, result));
    }

    fn sync_palette_selection(&mut self) {
        if !self.state.launcher_open {
            return;
        }
        let count = palette::filtered_entries(&self.state).len();
        palette::clamp_palette_selection(
            &mut self.state.launcher_selection,
            &mut self.state.launcher_scroll_offset,
            palette::PALETTE_RESULT_ROWS,
            count,
        );
    }

    fn sync_process_manager_visible_rows(&mut self) {
        let Some(window) = self
            .state
            .focused_window()
            .filter(|window| window.application == ApplicationKind::Processes)
        else {
            return;
        };
        let window_inner = ui::windows::content_inner(self.geometry.desktop, window, &self.state);
        let rows = window_inner.height.saturating_sub(4) as usize;
        let filter = self
            .state
            .process_manager(window.id)
            .map(|manager| manager.filter.clone());
        let match_count = match (&self.state.processes, &filter) {
            (Loadable::Ready(processes), Some(filter)) => {
                process_manager::matching_indices(processes, filter).len()
            }
            _ => 0,
        };
        if let Some(manager) = self.state.process_manager_mut(window.id) {
            manager.visible_rows = rows.max(1);
            manager.clamp_selection(match_count);
        }
    }

    fn sync_file_manager_visible_rows(&mut self) {
        let Some(window) = self
            .state
            .focused_window()
            .filter(|window| window.application == ApplicationKind::FileManager)
        else {
            return;
        };
        let window_inner = ui::windows::content_inner(self.geometry.desktop, window, &self.state);
        let layout = ui::file_manager::layout(window_inner);
        if let Some(manager) = self.state.file_manager_mut(window.id) {
            ui::file_manager::sync_visible_rows(manager, layout.list_rows);
            manager.clamp_selection();
        }
    }

    fn resize_focused_terminal(&mut self) {
        let Some(window) = self.state.focused_window() else {
            return;
        };
        if window.application != ApplicationKind::Terminal {
            return;
        }
        let window_id = window.id;
        let columns = self.geometry.desktop.width.saturating_sub(2);
        let rows = self.geometry.desktop.height.saturating_sub(2);
        let _ = self.terminal_manager.resize(window_id, columns, rows);
    }

    fn handle_terminal_event(&mut self, event: TerminalEvent) {
        match event {
            TerminalEvent::Output { window_id, bytes } => {
                if let Some(content) = self.terminal_manager.consume_output(window_id, &bytes) {
                    self.state.set_terminal_content(window_id, content);
                    self.state
                        .set_terminal_status(window_id, TerminalStatus::Running);
                }
            }
            TerminalEvent::Exited { window_id } => {
                self.terminal_manager.close(window_id);
                if self.state.terminal_status(window_id).is_some() {
                    self.state
                        .set_terminal_status(window_id, TerminalStatus::Exited);
                    self.state.status =
                        "Terminal process exited · press Ctrl+W to close".to_owned();
                }
            }
        }
    }

    async fn refresh_capabilities(&mut self) {
        let (system, processes) = tokio::join!(
            self.system_provider.snapshot(),
            self.process_provider.processes()
        );
        self.state.apply_event(Event::SystemInfoLoaded(
            system.map_err(|error| error.to_string()),
        ));
        self.state.apply_event(Event::ProcessesLoaded(
            processes.map_err(|error| error.to_string()),
        ));
        self.state.status = "Live · refreshed just now".to_owned();
    }
}

fn format_input_debug(key: crossterm::event::KeyEvent, dispatch: &input::KeyDispatch) -> String {
    use std::fmt::Write;

    let normalized = input::normalize::normalize_key_event(key);
    let dispatch_label = match dispatch {
        input::KeyDispatch::Action(action) => format!("Action({action:?})"),
        input::KeyDispatch::Terminal(bytes) => format!("Terminal({} bytes)", bytes.len()),
        input::KeyDispatch::Consumed => "Consumed".to_owned(),
    };
    let mut line = String::new();
    let _ = write!(
        line,
        "raw {:?}+{:?} → norm {:?}+{:?} → {}",
        key.code, key.modifiers, normalized.code, normalized.modifiers, dispatch_label
    );
    line
}

fn spawn_input_reader() -> UnboundedReceiver<Event> {
    let (sender, receiver) = mpsc::unbounded_channel();
    thread::spawn(move || {
        while let Ok(event) = event::read() {
            let event = match event {
                CrosstermEvent::Key(key) => Event::Key(key),
                CrosstermEvent::Mouse(mouse) => Event::Mouse(mouse),
                _ => continue,
            };
            if sender.send(event).is_err() {
                break;
            }
        }
    });
    receiver
}
