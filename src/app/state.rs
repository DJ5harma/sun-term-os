use std::collections::HashMap;
use std::path::PathBuf;

use crate::{
    app::file_manager::standard_places,
    domain::{Window, WindowId, WindowState, Workspace},
    machine::{DirectoryListing, ProcessInfo, SystemSnapshot},
};

pub use super::file_manager::{CreateKind, FileManagerFocus, FileSort, Place, SortColumn};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileManagerDialog {
    None,
    DeleteConfirm { path: PathBuf, label: String },
    Rename { path: PathBuf, input: String },
    Create { kind: CreateKind, input: String },
}
pub use super::process_manager::ProcessManagerState;

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
    pub dialog: FileManagerDialog,
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
            dialog: FileManagerDialog::None,
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
    pub workspaces: Vec<Workspace>,
    pub active_workspace: usize,
    pub launcher_open: bool,
    pub launcher_query: String,
    pub launcher_selection: usize,
    pub launcher_scroll_offset: usize,
    /// Updated each frame from launcher geometry (see `sync_palette_selection`).
    pub launcher_visible_rows: usize,
    pub next_window_id: WindowId,
    pub terminal_contents: HashMap<WindowId, String>,
    pub terminal_statuses: HashMap<WindowId, TerminalStatus>,
    pub file_managers: HashMap<WindowId, FileManagerState>,
    pub process_managers: HashMap<WindowId, ProcessManagerState>,
    pub system_info_views: HashMap<WindowId, Loadable<SystemSnapshot>>,
    pub system: Loadable<SystemSnapshot>,
    pub processes: Loadable<Vec<ProcessInfo>>,
    pub status: String,
    pub input_debug: bool,
    pub input_debug_line: String,
    /// After window-pick chord begins, next digit 1–9 focuses a window.
    pub window_pick_mode: bool,
    pub should_quit: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(3)
    }
}

impl AppState {
    pub fn new(workspace_count: usize) -> Self {
        let workspace_count = workspace_count.clamp(1, 9);
        let workspace_hint = if workspace_count == 1 {
            "F1 workspace".to_owned()
        } else {
            format!("F1–F{workspace_count} workspaces")
        };
        Self {
            workspaces: (0..workspace_count)
                .map(|id| Workspace {
                    id,
                    windows: Vec::new(),
                    focused_window: None,
                })
                .collect(),
            active_workspace: 0,
            launcher_open: false,
            launcher_query: String::new(),
            launcher_selection: 0,
            launcher_scroll_offset: 0,
            launcher_visible_rows: 8,
            next_window_id: 1,
            terminal_contents: HashMap::new(),
            terminal_statuses: HashMap::new(),
            file_managers: HashMap::new(),
            process_managers: HashMap::new(),
            system_info_views: HashMap::new(),
            system: Loadable::Loading,
            processes: Loadable::Loading,
            status: format!(
                "Welcome · Alt+P palette (! for shell) · Ctrl+G 1–9 windows · {workspace_hint}"
            ),
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

    pub fn host_label(&self) -> &str {
        match &self.system {
            Loadable::Ready(snapshot) => snapshot.hostname.as_str(),
            _ => "This machine",
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

    pub fn process_manager(&self, window_id: WindowId) -> Option<&ProcessManagerState> {
        self.process_managers.get(&window_id)
    }

    pub(crate) fn process_manager_mut(
        &mut self,
        window_id: WindowId,
    ) -> Option<&mut ProcessManagerState> {
        self.process_managers.get_mut(&window_id)
    }

    pub(crate) fn init_process_manager(&mut self, window_id: WindowId) {
        self.process_managers
            .insert(window_id, ProcessManagerState::new());
    }

    pub(crate) fn remove_process_manager(&mut self, window_id: WindowId) {
        self.process_managers.remove(&window_id);
    }

    pub fn system_info_view(&self, window_id: WindowId) -> Option<&Loadable<SystemSnapshot>> {
        self.system_info_views.get(&window_id)
    }

    pub(crate) fn init_system_info_view(&mut self, window_id: WindowId) {
        self.system_info_views.insert(window_id, Loadable::Loading);
    }

    pub(crate) fn remove_system_info_view(&mut self, window_id: WindowId) {
        self.system_info_views.remove(&window_id);
    }

    pub(crate) fn mark_capability_refresh_loading(&mut self) {
        self.system = Loadable::Loading;
        self.processes = Loadable::Loading;
        for view in self.system_info_views.values_mut() {
            *view = Loadable::Loading;
        }
        for manager in self.process_managers.values_mut() {
            manager.listing = Loadable::Loading;
        }
    }
}
