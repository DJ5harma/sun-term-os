use std::collections::HashMap;

use crate::{
    domain::{Window, WindowId, WindowState, Workspace},
    events::Event,
    machine::{MachineDescriptor, MachineId, MachineKind, ProcessInfo, SystemSnapshot},
};

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
    pub launcher_open: bool,
    pub launcher_selection: usize,
    pub next_window_id: WindowId,
    pub terminal_contents: HashMap<WindowId, String>,
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
                kind: MachineKind::Local,
            },
            workspaces: (0..3)
                .map(|id| Workspace {
                    id,
                    windows: Vec::new(),
                    focused_window: None,
                })
                .collect(),
            active_workspace: 0,
            launcher_open: false,
            launcher_selection: 0,
            next_window_id: 1,
            terminal_contents: HashMap::new(),
            system: Loadable::Loading,
            processes: Loadable::Loading,
            status: "Welcome to TDE · press p to open the launcher".to_owned(),
            should_quit: false,
        }
    }
}

impl AppState {
    pub fn current_workspace(&self) -> &Workspace {
        &self.workspaces[self.active_workspace]
    }

    pub fn focused_window(&self) -> Option<&Window> {
        let workspace = self.current_workspace();
        workspace
            .focused_window
            .and_then(|id| workspace.windows.iter().find(|window| window.id == id))
    }

    pub(crate) fn current_workspace_mut(&mut self) -> &mut Workspace {
        &mut self.workspaces[self.active_workspace]
    }

    pub(crate) fn focused_window_mut(&mut self) -> Option<&mut Window> {
        let focused = self.current_workspace().focused_window?;
        self.current_workspace_mut()
            .windows
            .iter_mut()
            .find(|window| window.id == focused)
    }

    pub(crate) fn apply_event(&mut self, event: Event) {
        match event {
            Event::SystemInfoLoaded(result) => {
                self.system = match result {
                    Ok(snapshot) => Loadable::Ready(snapshot),
                    Err(error) => Loadable::Failed(error),
                };
            }
            Event::ProcessesLoaded(result) => {
                self.processes = match result {
                    Ok(processes) => Loadable::Ready(processes),
                    Err(error) => Loadable::Failed(error),
                };
            }
            _ => {}
        }
    }

    pub(crate) fn visible_window_ids(&self) -> Vec<WindowId> {
        self.current_workspace()
            .windows
            .iter()
            .filter(|window| window.state != WindowState::Minimized)
            .map(|window| window.id)
            .collect()
    }

    pub fn terminal_content(&self, window_id: WindowId) -> &str {
        self.terminal_contents
            .get(&window_id)
            .map(String::as_str)
            .unwrap_or("")
    }

    pub(crate) fn set_terminal_content(&mut self, window_id: WindowId, content: String) {
        self.terminal_contents.insert(window_id, content);
    }

    pub(crate) fn remove_terminal_content(&mut self, window_id: WindowId) {
        self.terminal_contents.remove(&window_id);
    }
}
