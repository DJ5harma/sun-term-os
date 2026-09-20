use std::collections::HashMap;
use std::path::PathBuf;

use crate::{
    app::{file_manager::standard_places, terminal_view::TerminalViewState},
    config::Config,
    domain::{Window, WindowId, WindowState, Workspace},
    machine::{
        DirectoryListing, MachineId, ProcessInfo, SystemSnapshot, registry::ConnectionState,
    },
};

pub use super::file_manager::{CreateKind, FileManagerFocus, FileSort, Place, SortColumn};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileManagerDialog {
    None,
    DeleteConfirm { path: PathBuf, label: String },
    Rename { path: PathBuf, input: String },
    Create { kind: CreateKind, input: String },
    GoToPath { input: String },
}
pub use super::home_screen::HomeScreenState;
pub use super::launcher::LauncherState;
pub use super::machines::MachinesState;
pub use super::process_manager::ProcessManagerState;
pub use super::services_manager::ServicesState;
pub use super::settings::SettingsState;
pub use super::text_viewer::TextViewerState;

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
    pub terminal_views: HashMap<WindowId, TerminalViewState>,
    pub terminal_statuses: HashMap<WindowId, TerminalStatus>,
    /// Unix seconds when system/process data last refreshed successfully.
    pub capabilities_refreshed_at: Option<u64>,
    pub active_machine_id: MachineId,
    pub systems: HashMap<MachineId, Loadable<SystemSnapshot>>,
    pub process_lists: HashMap<MachineId, Loadable<Vec<ProcessInfo>>>,
    pub machine_connections: HashMap<String, ConnectionState>,
    /// Updated each frame from desktop geometry (terminal scrollback).
    pub terminal_body_rows: usize,
    pub file_managers: HashMap<WindowId, FileManagerState>,
    pub process_managers: HashMap<WindowId, ProcessManagerState>,
    pub system_info_views: HashMap<WindowId, Loadable<SystemSnapshot>>,
    pub machines_views: HashMap<WindowId, MachinesState>,
    pub launcher_views: HashMap<WindowId, LauncherState>,
    pub text_viewers: HashMap<WindowId, TextViewerState>,
    pub services_views: HashMap<WindowId, ServicesState>,
    pub status: String,
    pub input_debug: bool,
    pub input_debug_line: String,
    /// After window-pick chord begins, next digit 1–9 focuses a window.
    pub window_pick_mode: bool,
    pub should_quit: bool,
    pub config: Config,
    pub settings_views: HashMap<WindowId, SettingsState>,
    pub home_screens: Vec<HomeScreenState>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let config = config.normalized();
        let workspace_count = config.workspace_count;
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
                    show_desktop: false,
                    show_desktop_restore_focus: None,
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
            terminal_views: HashMap::new(),
            terminal_statuses: HashMap::new(),
            capabilities_refreshed_at: None,
            active_machine_id: MachineId::Local,
            systems: HashMap::new(),
            process_lists: HashMap::new(),
            machine_connections: HashMap::new(),
            terminal_body_rows: 20,
            file_managers: HashMap::new(),
            process_managers: HashMap::new(),
            system_info_views: HashMap::new(),
            machines_views: HashMap::new(),
            launcher_views: HashMap::new(),
            text_viewers: HashMap::new(),
            services_views: HashMap::new(),
            status: format!(
                "Welcome · Alt+P palette (! for shell) · Ctrl+G 1–9 windows · {workspace_hint}"
            ),
            input_debug: false,
            input_debug_line: String::new(),
            window_pick_mode: false,
            should_quit: false,
            config,
            settings_views: HashMap::new(),
            home_screens: (0..workspace_count)
                .map(|_| HomeScreenState::default())
                .collect(),
        }
    }

    pub fn settings_view(&self, window_id: WindowId) -> Option<&SettingsState> {
        self.settings_views.get(&window_id)
    }

    pub(crate) fn init_settings_view(&mut self, window_id: WindowId) {
        let theme_preset_index = crate::app::theme_presets::index_for_theme(&self.config.theme);
        self.settings_views.insert(
            window_id,
            SettingsState {
                selected: 0,
                theme_preset_index,
            },
        );
    }

    pub(crate) fn remove_settings_view(&mut self, window_id: WindowId) {
        self.settings_views.remove(&window_id);
    }

    pub(crate) fn settings_view_mut(&mut self, window_id: WindowId) -> Option<&mut SettingsState> {
        self.settings_views.get_mut(&window_id)
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

    pub fn host_label(&self) -> String {
        self.host_label_for(&self.active_machine_id)
    }

    pub fn host_label_for(&self, machine_id: &MachineId) -> String {
        if let Some(Loadable::Ready(snapshot)) = self.systems.get(machine_id) {
            return snapshot.hostname.clone();
        }
        match machine_id {
            MachineId::Local => "Local".to_owned(),
            MachineId::Named(id) => self
                .config
                .machines
                .iter()
                .find(|profile| profile.id == *id)
                .map(|profile| profile.display_label().to_owned())
                .unwrap_or_else(|| id.clone()),
        }
    }

    pub fn window_machine_id(&self, window_id: WindowId) -> Option<MachineId> {
        for workspace in &self.workspaces {
            if let Some(window) = workspace
                .windows
                .iter()
                .find(|window| window.id == window_id)
            {
                return Some(window.machine_id.clone());
            }
        }
        None
    }

    pub fn system_for(&self, machine_id: &MachineId) -> Loadable<SystemSnapshot> {
        self.systems
            .get(machine_id)
            .cloned()
            .unwrap_or(Loadable::Loading)
    }

    pub fn processes_for(&self, machine_id: &MachineId) -> Loadable<Vec<ProcessInfo>> {
        self.process_lists
            .get(machine_id)
            .cloned()
            .unwrap_or(Loadable::Loading)
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
        self.terminal_contents.insert(window_id, content.clone());
        let view = self.terminal_views.entry(window_id).or_default();
        let follow = view.follow_output();
        view.update_screen(&content);
        if follow {
            view.reset_scroll();
        }
    }

    pub(crate) fn set_terminal_title(&mut self, window_id: WindowId, title: String) {
        self.terminal_views.entry(window_id).or_default().title = Some(title);
    }

    pub fn terminal_view(&self, window_id: WindowId) -> Option<&TerminalViewState> {
        self.terminal_views.get(&window_id)
    }

    pub(crate) fn terminal_view_mut(&mut self, window_id: WindowId) -> &mut TerminalViewState {
        self.terminal_views.entry(window_id).or_default()
    }

    pub(crate) fn remove_terminal_content(&mut self, window_id: WindowId) {
        self.terminal_contents.remove(&window_id);
        self.terminal_views.remove(&window_id);
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

    pub(crate) fn init_process_manager(
        &mut self,
        window_id: WindowId,
        listing: Loadable<Vec<ProcessInfo>>,
    ) {
        let mut manager = ProcessManagerState::new();
        manager.listing = listing;
        self.process_managers.insert(window_id, manager);
    }

    pub(crate) fn remove_process_manager(&mut self, window_id: WindowId) {
        self.process_managers.remove(&window_id);
    }

    pub fn system_info_view(&self, window_id: WindowId) -> Option<&Loadable<SystemSnapshot>> {
        self.system_info_views.get(&window_id)
    }

    pub(crate) fn init_system_info_view(
        &mut self,
        window_id: WindowId,
        listing: Loadable<SystemSnapshot>,
    ) {
        self.system_info_views.insert(window_id, listing);
    }

    pub(crate) fn remove_system_info_view(&mut self, window_id: WindowId) {
        self.system_info_views.remove(&window_id);
    }

    pub(crate) fn mark_capability_refresh_loading(&mut self) {
        for listing in self.systems.values_mut() {
            *listing = Loadable::Loading;
        }
        for listing in self.process_lists.values_mut() {
            *listing = Loadable::Loading;
        }
        for view in self.system_info_views.values_mut() {
            *view = Loadable::Loading;
        }
        for manager in self.process_managers.values_mut() {
            manager.listing = Loadable::Loading;
        }
    }

    pub(crate) fn init_machines_view(&mut self, window_id: WindowId) {
        self.machines_views.insert(window_id, MachinesState::new());
    }

    pub(crate) fn remove_machines_view(&mut self, window_id: WindowId) {
        self.machines_views.remove(&window_id);
    }

    pub fn machines_view(&self, window_id: WindowId) -> Option<&MachinesState> {
        self.machines_views.get(&window_id)
    }

    pub(crate) fn machines_view_mut(&mut self, window_id: WindowId) -> Option<&mut MachinesState> {
        self.machines_views.get_mut(&window_id)
    }

    pub(crate) fn init_launcher_view(&mut self, window_id: WindowId) {
        self.launcher_views.insert(window_id, LauncherState::new());
    }

    pub(crate) fn remove_launcher_view(&mut self, window_id: WindowId) {
        self.launcher_views.remove(&window_id);
    }

    pub fn launcher_view(&self, window_id: WindowId) -> Option<&LauncherState> {
        self.launcher_views.get(&window_id)
    }

    pub(crate) fn launcher_view_mut(&mut self, window_id: WindowId) -> Option<&mut LauncherState> {
        self.launcher_views.get_mut(&window_id)
    }

    pub(crate) fn init_text_viewer(&mut self, window_id: WindowId, path: PathBuf) {
        self.text_viewers
            .insert(window_id, TextViewerState::new(path));
    }

    pub(crate) fn init_text_viewer_empty(&mut self, window_id: WindowId) {
        self.text_viewers
            .insert(window_id, TextViewerState::new_empty());
    }

    pub(crate) fn remove_text_viewer(&mut self, window_id: WindowId) {
        self.text_viewers.remove(&window_id);
    }

    pub fn text_viewer(&self, window_id: WindowId) -> Option<&TextViewerState> {
        self.text_viewers.get(&window_id)
    }

    pub(crate) fn text_viewer_mut(&mut self, window_id: WindowId) -> Option<&mut TextViewerState> {
        self.text_viewers.get_mut(&window_id)
    }

    pub(crate) fn init_services_view(&mut self, window_id: WindowId) {
        self.services_views.insert(window_id, ServicesState::new());
    }

    pub(crate) fn remove_services_view(&mut self, window_id: WindowId) {
        self.services_views.remove(&window_id);
    }

    pub fn services_view(&self, window_id: WindowId) -> Option<&ServicesState> {
        self.services_views.get(&window_id)
    }

    pub(crate) fn services_view_mut(&mut self, window_id: WindowId) -> Option<&mut ServicesState> {
        self.services_views.get_mut(&window_id)
    }
}
