use std::collections::HashMap;
use std::path::PathBuf;

use crate::{
    app::file_manager::standard_places,
    domain::{Window, WindowId, WindowState, Workspace},
    events::Event,
    machine::{
        DirectoryListing, MachineDescriptor, MachineId, MachineKind, ProcessInfo, SystemSnapshot,
    },
};

pub use super::file_manager::{FileManagerFocus, FileSort, Place, SortColumn};

#[derive(Debug, Clone, PartialEq)]
pub enum Loadable<T> {
    Loading,
    Ready(T),
    Failed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileManagerState {
    pub current_path: PathBuf,
    pub show_hidden: bool,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub focus: FileManagerFocus,
    pub selected_place: usize,
    pub sort: FileSort,
    pub places: Vec<Place>,
    pub history: Vec<PathBuf>,
    pub history_index: usize,
    pub listing: Loadable<DirectoryListing>,
    pub visible_rows: usize,
}

impl FileManagerState {
    pub fn new(initial_path: PathBuf) -> Self {
        Self {
            current_path: initial_path.clone(),
            show_hidden: false,
            selected_index: 0,
            scroll_offset: 0,
            focus: FileManagerFocus::List,
            selected_place: 0,
            sort: FileSort::default(),
            places: standard_places(),
            history: vec![initial_path],
            history_index: 0,
            listing: Loadable::Loading,
            visible_rows: 1,
        }
    }

    pub fn clamp_selection(&mut self) {
        super::file_manager::clamp_listing_selection(
            &self.listing,
            &self.current_path,
            &mut self.selected_index,
            &mut self.scroll_offset,
            self.visible_rows,
        );
        if !self.places.is_empty() {
            self.selected_place = self.selected_place.min(self.places.len() - 1);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalStatus {
    Starting,
    Running,
    Exited,
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
    pub terminal_statuses: HashMap<WindowId, TerminalStatus>,
    pub file_managers: HashMap<WindowId, FileManagerState>,
    pub system: Loadable<SystemSnapshot>,
    pub processes: Loadable<Vec<ProcessInfo>>,
    pub status: String,
    pub input_debug: bool,
    pub input_debug_line: String,
    /// After [Action::BeginWindowPick], next digit 1–9 focuses a window.
    pub window_pick_mode: bool,
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
            terminal_statuses: HashMap::new(),
            file_managers: HashMap::new(),
            system: Loadable::Loading,
            processes: Loadable::Loading,
            status: "Welcome · ⊞ Apps · Ctrl+G then 1–9 for windows · F1–F3 workspaces".to_owned(),
            input_debug: false,
            input_debug_line: String::new(),
            window_pick_mode: false,
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
            Event::DirectoryLoaded(window_id, result) => {
                if let Some(manager) = self.file_managers.get_mut(&window_id) {
                    manager.listing = match result {
                        Ok(listing) => {
                            manager.current_path = listing.path.clone();
                            Loadable::Ready(super::file_manager::apply_sorted_listing(
                                listing,
                                manager.sort,
                            ))
                        }
                        Err(error) => Loadable::Failed(error),
                    };
                    manager.clamp_selection();
                }
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

    pub fn terminal_status(&self, window_id: WindowId) -> Option<&TerminalStatus> {
        self.terminal_statuses.get(&window_id)
    }

    pub(crate) fn set_terminal_status(&mut self, window_id: WindowId, status: TerminalStatus) {
        self.terminal_statuses.insert(window_id, status);
    }

    pub(crate) fn remove_terminal_status(&mut self, window_id: WindowId) {
        self.terminal_statuses.remove(&window_id);
    }

    pub fn file_manager(&self, window_id: WindowId) -> Option<&FileManagerState> {
        self.file_managers.get(&window_id)
    }

    pub(crate) fn file_manager_mut(
        &mut self,
        window_id: WindowId,
    ) -> Option<&mut FileManagerState> {
        self.file_managers.get_mut(&window_id)
    }

    pub(crate) fn init_file_manager(&mut self, window_id: WindowId, initial_path: PathBuf) {
        self.file_managers
            .insert(window_id, FileManagerState::new(initial_path));
    }

    pub(crate) fn remove_file_manager(&mut self, window_id: WindowId) {
        self.file_managers.remove(&window_id);
    }
}
