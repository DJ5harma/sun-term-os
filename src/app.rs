use std::{sync::Arc, time::Duration};

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;
use tokio::time::{self, MissedTickBehavior};

use crate::{
    actions::Action,
    events::Event,
    machine::{
        MachineDescriptor, MachineId, ProcessInfo, ProcessProvider, SystemInfoProvider,
        SystemSnapshot,
    },
    ui,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Overview,
    System,
    Processes,
}

impl Panel {
    pub const ALL: [Self; 3] = [Self::Overview, Self::System, Self::Processes];

    pub fn title(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::System => "System",
            Self::Processes => "Processes",
        }
    }

    fn next(self) -> Self {
        let index = Self::ALL
            .iter()
            .position(|panel| *panel == self)
            .unwrap_or(0);
        Self::ALL[(index + 1) % Self::ALL.len()]
    }

    fn previous(self) -> Self {
        let index = Self::ALL
            .iter()
            .position(|panel| *panel == self)
            .unwrap_or(0);
        Self::ALL[(index + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Loadable<T> {
    Loading,
    Ready(T),
    Failed(String),
}

#[derive(Debug)]
pub struct AppState {
    pub machine: MachineDescriptor,
    pub workspaces: Vec<Workspace>,
    pub active_workspace: usize,
    pub focused_panel: Panel,
    pub palette_open: bool,
    pub palette_selection: usize,
    pub system: Loadable<SystemSnapshot>,
    pub processes: Loadable<Vec<ProcessInfo>>,
    pub status: String,
    pub should_quit: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            machine: MachineDescriptor {
                id: MachineId("local".to_owned()),
                name: "This machine".to_owned(),
                kind: crate::machine::MachineKind::Local,
            },
            workspaces: vec![
                Workspace {
                    name: "Main".to_owned(),
                },
                Workspace {
                    name: "Monitor".to_owned(),
                },
                Workspace {
                    name: "Scratch".to_owned(),
                },
            ],
            active_workspace: 0,
            focused_panel: Panel::Overview,
            palette_open: false,
            palette_selection: 0,
            system: Loadable::Loading,
            processes: Loadable::Loading,
            status: "Starting local capabilities…".to_owned(),
            should_quit: false,
        }
    }
}

pub struct App {
    pub state: AppState,
    system_provider: Arc<dyn SystemInfoProvider>,
    process_provider: Arc<dyn ProcessProvider>,
}

impl App {
    pub fn new(
        system_provider: Arc<dyn SystemInfoProvider>,
        process_provider: Arc<dyn ProcessProvider>,
    ) -> Self {
        Self {
            state: AppState::default(),
            system_provider,
            process_provider,
        }
    }

    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let mut ticker = time::interval(Duration::from_secs(3));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        self.refresh().await;

        while !self.state.should_quit {
            terminal.draw(|frame| ui::render(frame, &self.state))?;

            tokio::select! {
                _ = ticker.tick() => self.handle_event(Event::Tick).await,
                event = Self::read_event() => {
                    if let Some(event) = event? {
                        self.handle_event(event).await;
                    }
                }
            }
        }
        Ok(())
    }

    async fn read_event() -> Result<Option<Event>> {
        let event = tokio::task::spawn_blocking(|| -> std::io::Result<Option<Event>> {
            if event::poll(Duration::from_millis(100))? {
                Ok(Some(match event::read()? {
                    CrosstermEvent::Key(key) => Event::Key(key),
                    _ => return Ok(None),
                }))
            } else {
                Ok(None)
            }
        })
        .await??;
        Ok(event)
    }

    async fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => {
                if let Some(action) = self.action_for_key(key) {
                    self.dispatch(action).await;
                }
            }
            Event::SystemInfoLoaded(result) => match result {
                Ok(snapshot) => self.state.system = Loadable::Ready(snapshot),
                Err(error) => self.state.system = Loadable::Failed(error),
            },
            Event::ProcessesLoaded(result) => match result {
                Ok(processes) => self.state.processes = Loadable::Ready(processes),
                Err(error) => self.state.processes = Loadable::Failed(error),
            },
            Event::Tick => self.refresh().await,
        }
    }

    async fn dispatch(&mut self, action: Action) {
        match action {
            Action::Quit => self.state.should_quit = true,
            Action::Refresh => self.refresh().await,
            Action::ToggleCommandPalette => {
                self.state.palette_open = !self.state.palette_open;
                self.state.palette_selection = 0;
            }
            Action::CloseCommandPalette => self.state.palette_open = false,
            Action::MovePaletteUp => {
                self.state.palette_selection = self.state.palette_selection.saturating_sub(1)
            }
            Action::MovePaletteDown => {
                self.state.palette_selection = (self.state.palette_selection + 1).min(3)
            }
            Action::ExecutePaletteSelection => {
                self.state.palette_open = false;
                match self.state.palette_selection {
                    0 => self.refresh().await,
                    1 | 2 => {
                        let index = self.state.palette_selection - 1;
                        if index < self.state.workspaces.len() {
                            self.state.active_workspace = index;
                            self.state.status =
                                format!("Workspace: {}", self.state.workspaces[index].name);
                        }
                    }
                    _ => self.state.should_quit = true,
                }
            }
            Action::FocusNext => self.state.focused_panel = self.state.focused_panel.next(),
            Action::FocusPrevious => self.state.focused_panel = self.state.focused_panel.previous(),
            Action::SwitchWorkspace(index) if index < self.state.workspaces.len() => {
                self.state.active_workspace = index;
                self.state.status = format!("Workspace: {}", self.state.workspaces[index].name);
            }
            Action::SwitchWorkspace(_) => self.state.status = "Workspace does not exist".to_owned(),
        }
    }

    async fn refresh(&mut self) {
        self.state.system = Loadable::Loading;
        self.state.processes = Loadable::Loading;
        self.state.status = "Refreshing local capabilities…".to_owned();

        let (system, processes) = tokio::join!(
            self.system_provider.snapshot(),
            self.process_provider.processes()
        );
        self.apply_event(Event::SystemInfoLoaded(
            system.map_err(|error| error.to_string()),
        ));
        self.apply_event(Event::ProcessesLoaded(
            processes.map_err(|error| error.to_string()),
        ));
        self.state.status = "Live · refreshed just now".to_owned();
    }

    fn apply_event(&mut self, event: Event) {
        match event {
            Event::SystemInfoLoaded(result) => {
                self.state.system = match result {
                    Ok(snapshot) => Loadable::Ready(snapshot),
                    Err(error) => Loadable::Failed(error),
                };
            }
            Event::ProcessesLoaded(result) => {
                self.state.processes = match result {
                    Ok(processes) => Loadable::Ready(processes),
                    Err(error) => Loadable::Failed(error),
                };
            }
            _ => {}
        }
    }

    fn action_for_key(&self, key: KeyEvent) -> Option<Action> {
        if self.state.palette_open {
            return match key.code {
                KeyCode::Esc => Some(Action::CloseCommandPalette),
                KeyCode::Up => Some(Action::MovePaletteUp),
                KeyCode::Down => Some(Action::MovePaletteDown),
                KeyCode::Enter => Some(Action::ExecutePaletteSelection),
                _ => None,
            };
        }

        match key {
            KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }
            | KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => Some(Action::Quit),
            KeyEvent {
                code: KeyCode::Char('p'),
                ..
            }
            | KeyEvent {
                code: KeyCode::Char(':'),
                ..
            } => Some(Action::ToggleCommandPalette),
            KeyEvent {
                code: KeyCode::Tab, ..
            } => Some(Action::FocusNext),
            KeyEvent {
                code: KeyCode::BackTab,
                ..
            } => Some(Action::FocusPrevious),
            KeyEvent {
                code: KeyCode::Char('r'),
                ..
            } => Some(Action::Refresh),
            KeyEvent {
                code: KeyCode::Char(number @ '1'..='3'),
                ..
            } => Some(Action::SwitchWorkspace(number as usize - '1' as usize)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use crate::machine::local::{LocalProcessProvider, LocalSystemInfoProvider};

    use super::*;

    fn test_app() -> App {
        App::new(
            Arc::new(LocalSystemInfoProvider),
            Arc::new(LocalProcessProvider),
        )
    }

    #[tokio::test]
    async fn invalid_workspace_does_not_change_active_workspace() {
        let mut app = test_app();
        app.dispatch(Action::SwitchWorkspace(99)).await;
        assert_eq!(app.state.active_workspace, 0);
        assert_eq!(app.state.status, "Workspace does not exist");
    }

    #[test]
    fn panel_order_is_cyclic() {
        assert_eq!(
            Panel::ALL,
            [Panel::Overview, Panel::System, Panel::Processes]
        );
    }

    #[test]
    fn capability_results_update_loadable_state() {
        let mut app = test_app();
        app.apply_event(Event::SystemInfoLoaded(Err("offline".to_owned())));
        app.apply_event(Event::ProcessesLoaded(Ok(Vec::new())));
        assert_eq!(app.state.system, Loadable::Failed("offline".to_owned()));
        assert_eq!(app.state.processes, Loadable::Ready(Vec::new()));
    }

    #[test]
    fn palette_input_is_translated_to_semantic_actions() {
        let mut app = test_app();
        app.state.palette_open = true;
        assert_eq!(
            app.action_for_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            Some(Action::MovePaletteDown)
        );
        assert_eq!(
            app.action_for_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            Some(Action::ExecutePaletteSelection)
        );
    }
}
