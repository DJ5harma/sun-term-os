mod effects;
mod sync;

pub use effects::EffectExecutor;
pub use sync::{
    sync_file_manager_visible_rows, sync_palette_selection, sync_process_manager_visible_rows,
};

use std::{thread, time::Duration};

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent};
use ratatui::DefaultTerminal;
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tokio::time::{self, MissedTickBehavior};

use crate::{
    actions::{Action, ShellAction},
    app::{
        ApplicationKind, reducer,
        state::{AppState, TerminalStatus},
    },
    config::Config,
    events::Event,
    input,
    machine::Machine,
    terminal::{TerminalEvent, TerminalManager},
    ui,
};

pub struct AppRuntime {
    pub state: AppState,
    refresh_interval_secs: u64,
    geometry: ui::geometry::UiGeometry,
    mouse_click: input::DoubleClickState,
    interactions: ui::interaction::InteractionMap,
    executor: EffectExecutor,
}

impl AppRuntime {
    pub fn new(config: Config, machine: Machine) -> Self {
        let config = config.normalized();
        Self {
            state: AppState::new(config.workspace_count),
            refresh_interval_secs: config.refresh_interval_secs,
            geometry: ui::geometry::UiGeometry::default(),
            mouse_click: input::DoubleClickState::default(),
            interactions: ui::interaction::InteractionMap::default(),
            executor: EffectExecutor {
                machine,
                terminal_manager: TerminalManager::default(),
            },
        }
    }

    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let mut ticker = time::interval(Duration::from_secs(self.refresh_interval_secs));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        let mut input_events = spawn_input_reader();
        self.dispatch(Action::Shell(ShellAction::Refresh)).await;

        while !self.state.should_quit {
            let size = terminal.size()?;
            self.geometry = ui::geometry::calculate(
                ratatui::layout::Rect::new(0, 0, size.width, size.height),
                &self.state,
            );
            sync_file_manager_visible_rows(&mut self.state, &self.geometry);
            sync_process_manager_visible_rows(&mut self.state, &self.geometry);
            sync_palette_selection(&mut self.state, &self.geometry);
            self.resize_focused_terminal();
            self.interactions.clear();
            terminal.draw(|frame| {
                ui::render(frame, &self.state, &self.geometry, &mut self.interactions)
            })?;

            tokio::select! {
                _ = ticker.tick() => self.handle_event(Event::Tick).await,
                terminal_event = self.executor.terminal_manager.events.recv() => {
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
                let dispatch = input::handle_key(key, &self.state);
                if self.state.input_debug {
                    self.state.input_debug_line = format_input_debug(key, &dispatch);
                }
                match dispatch {
                    input::KeyDispatch::Action(action) => self.dispatch(action).await,
                    input::KeyDispatch::Terminal(bytes) => {
                        if let Some(window_id) = terminal_window {
                            let _ = self
                                .executor
                                .terminal_manager
                                .write_input(window_id, &bytes);
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
            Event::Tick => self.dispatch(Action::Shell(ShellAction::Refresh)).await,
        }
    }

    async fn dispatch(&mut self, action: Action) {
        let effects = reducer::reduce(&mut self.state, action);
        self.executor.run(&mut self.state, effects).await;
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
        let _ = self
            .executor
            .terminal_manager
            .resize(window_id, columns, rows);
    }

    fn handle_terminal_event(&mut self, event: TerminalEvent) {
        match event {
            TerminalEvent::Output { window_id, bytes } => {
                if let Some(content) = self
                    .executor
                    .terminal_manager
                    .consume_output(window_id, &bytes)
                {
                    self.state.set_terminal_content(window_id, content);
                    self.state
                        .set_terminal_status(window_id, TerminalStatus::Running);
                }
            }
            TerminalEvent::Exited { window_id } => {
                self.executor.terminal_manager.close(window_id);
                if self.state.terminal_status(window_id).is_some() {
                    self.state
                        .set_terminal_status(window_id, TerminalStatus::Exited);
                    self.state.status =
                        "Terminal process exited · press Ctrl+W to close".to_owned();
                }
            }
        }
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
