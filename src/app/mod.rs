mod effects;
pub mod file_manager;
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
    events::Event,
    input,
    machine::{FilesystemProvider, ProcessProvider, SystemInfoProvider},
    terminal::{TerminalEvent, TerminalManager},
    ui,
};

use effects::Effect;

pub struct App {
    pub state: AppState,
    geometry: ui::geometry::UiGeometry,
    mouse_click: input::DoubleClickState,
    terminal_manager: TerminalManager,
    system_provider: Arc<dyn SystemInfoProvider>,
    process_provider: Arc<dyn ProcessProvider>,
    filesystem_provider: Arc<dyn FilesystemProvider>,
}

impl App {
    pub fn new(
        system_provider: Arc<dyn SystemInfoProvider>,
        process_provider: Arc<dyn ProcessProvider>,
        filesystem_provider: Arc<dyn FilesystemProvider>,
    ) -> Self {
        Self {
            state: AppState::default(),
            geometry: ui::geometry::UiGeometry::default(),
            mouse_click: input::DoubleClickState::default(),
            terminal_manager: TerminalManager::default(),
            system_provider,
            process_provider,
            filesystem_provider,
        }
    }

    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let mut ticker = time::interval(Duration::from_secs(3));
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
            self.resize_focused_terminal();
            terminal.draw(|frame| ui::render(frame, &self.state, &self.geometry))?;

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
                let file_manager_focused = self
                    .state
                    .focused_window()
                    .is_some_and(|window| window.application == ApplicationKind::FileManager);
                if let Some(action) = input::action_for_key(
                    key,
                    self.state.launcher_open,
                    terminal_window.is_some(),
                    file_manager_focused,
                ) {
                    self.dispatch(action).await;
                } else if let Some(window_id) = terminal_window
                    && !self.state.launcher_open
                    && let Some(input) = input::terminal_input(key)
                {
                    let _ = self.terminal_manager.write_input(window_id, &input);
                }
            }
            Event::Mouse(mouse) => {
                let actions = input::actions_for_mouse(
                    mouse,
                    &self.state,
                    &self.geometry,
                    &mut self.mouse_click,
                );
                let handled = !actions.is_empty();
                for action in actions {
                    self.dispatch(action).await;
                }
                if !handled
                    && !self.state.launcher_open
                    && let Some(window_id) = self
                        .state
                        .focused_window()
                        .filter(|window| window.application == ApplicationKind::Terminal)
                        .map(|window| window.id)
                    && let Some(input) = input::terminal_mouse(mouse, &self.geometry)
                {
                    let _ = self.terminal_manager.write_input(window_id, &input);
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
            }
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
        let window_inner = ui::windows::content_inner(self.geometry.desktop);
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
