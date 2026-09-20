use std::path::PathBuf;

use crate::{
    actions::Action,
    app::{AppState, ApplicationKind, Loadable, TerminalStatus, Window, WindowState},
    machine::{FileEntryKind, local::default_start_path},
};

use super::effects::Effect;

pub fn reduce(state: &mut AppState, action: Action) -> Vec<Effect> {
    match action {
        Action::Quit => state.should_quit = true,
        Action::Refresh => {
            state.system = Loadable::Loading;
            state.processes = Loadable::Loading;
            state.status = "Refreshing local capabilities…".to_owned();
            return vec![Effect::RefreshCapabilities];
        }
        Action::ToggleLauncher => {
            state.launcher_open = !state.launcher_open;
            state.launcher_selection = 0;
        }
        Action::CloseLauncher => state.launcher_open = false,
        Action::MoveLauncherUp => {
            state.launcher_selection = state.launcher_selection.saturating_sub(1)
        }
        Action::MoveLauncherDown => {
            state.launcher_selection =
                (state.launcher_selection + 1).min(ApplicationKind::ALL.len() - 1)
        }
        Action::ExecuteLauncherSelection => {
            state.launcher_open = false;
            if let Some(application) = ApplicationKind::ALL.get(state.launcher_selection) {
                let window_id = open_application(state, *application);
                return open_application_effects(state, *application, window_id);
            }
        }
        Action::OpenApplication(application) => {
            state.launcher_open = false;
            let window_id = open_application(state, application);
            return open_application_effects(state, application, window_id);
        }
        Action::CloseWindow => {
            if let Some((window_id, application)) = close_focused_window(state) {
                match application {
                    ApplicationKind::Terminal => {
                        return vec![Effect::StopTerminal(window_id)];
                    }
                    ApplicationKind::FileManager => {
                        state.remove_file_manager(window_id);
                    }
                    _ => {}
                }
            }
        }
        Action::FocusNextWindow => focus_window_by_offset(state, 1),
        Action::FocusPreviousWindow => focus_window_by_offset(state, -1),
        Action::FocusWindow(id) => focus_window(state, id),
        Action::MinimizeWindow => {
            if let Some(window) = state.focused_window_mut() {
                window.state = WindowState::Minimized;
            }
            focus_window_by_offset(state, 1);
        }
        Action::ToggleMaximizeWindow => {
            if let Some(window) = state.focused_window_mut() {
                window.state = match window.state {
                    WindowState::Maximized => WindowState::Normal,
                    _ => WindowState::Maximized,
                };
            }
        }
        Action::SwitchWorkspace(index) if index < state.workspaces.len() => {
            state.active_workspace = index;
            state.status = format!("Workspace {}", index + 1);
        }
        Action::SwitchWorkspace(_) => state.status = "Workspace does not exist".to_owned(),
        Action::MoveFileSelection(offset) => {
            if let Some(window_id) = focused_file_manager_window(state)
                && let Some(manager) = state.file_manager_mut(window_id)
                && let Loadable::Ready(listing) = &manager.listing
                && !listing.entries.is_empty()
            {
                let count = listing.entries.len();
                let next = manager.selected_index as i32 + offset;
                manager.selected_index = next.clamp(0, count as i32 - 1) as usize;
            }
        }
        Action::OpenSelectedEntry => return open_selected_entry(state),
        Action::FileManagerParent => {
            if let Some(window_id) = focused_file_manager_window(state)
                && let Some(manager) = state.file_manager(window_id)
            {
                if let Some(parent) = manager.current_path.parent()
                    && parent != manager.current_path.as_path()
                {
                    return navigate_file_manager(state, window_id, parent.to_path_buf());
                }
                state.status = "Already at the root directory".to_owned();
            }
        }
        Action::ToggleFileManagerHidden => {
            if let Some(window_id) = focused_file_manager_window(state)
                && let Some(manager) = state.file_manager_mut(window_id)
            {
                manager.show_hidden = !manager.show_hidden;
                let path = manager.current_path.clone();
                return navigate_file_manager(state, window_id, path);
            }
        }
        Action::ReloadFileManager => return reload_focused_file_manager(state),
        Action::SelectFileManagerRow(window_id, index) => {
            if focused_file_manager_window(state) == Some(window_id)
                && let Some(manager) = state.file_manager_mut(window_id)
                && let Loadable::Ready(listing) = &manager.listing
                && index < listing.entries.len()
            {
                manager.selected_index = index;
            }
        }
    }
    Vec::new()
}

fn focused_file_manager_window(state: &AppState) -> Option<u64> {
    state
        .focused_window()
        .filter(|window| window.application == ApplicationKind::FileManager)
        .map(|window| window.id)
}

fn open_application_effects(
    state: &mut AppState,
    application: ApplicationKind,
    window_id: u64,
) -> Vec<Effect> {
    match application {
        ApplicationKind::Terminal => vec![Effect::StartTerminal(window_id)],
        ApplicationKind::FileManager => {
            let path = default_start_path();
            state.init_file_manager(window_id, path.clone());
            vec![Effect::ReadDirectory(window_id, path)]
        }
        _ => Vec::new(),
    }
}

fn navigate_file_manager(state: &mut AppState, window_id: u64, path: PathBuf) -> Vec<Effect> {
    if let Some(manager) = state.file_manager_mut(window_id) {
        manager.current_path = path.clone();
        manager.listing = Loadable::Loading;
        manager.selected_index = 0;
        vec![Effect::ReadDirectory(window_id, path)]
    } else {
        Vec::new()
    }
}

fn reload_focused_file_manager(state: &mut AppState) -> Vec<Effect> {
    if let Some(window_id) = focused_file_manager_window(state)
        && let Some(manager) = state.file_manager(window_id)
    {
        let path = manager.current_path.clone();
        state.status = format!("Reading {}", path.display());
        return navigate_file_manager(state, window_id, path);
    }
    Vec::new()
}

fn open_selected_entry(state: &mut AppState) -> Vec<Effect> {
    let Some(window_id) = focused_file_manager_window(state) else {
        return Vec::new();
    };
    let Some(manager) = state.file_manager(window_id) else {
        return Vec::new();
    };
    let Loadable::Ready(listing) = &manager.listing else {
        return Vec::new();
    };
    let Some(entry) = listing.entries.get(manager.selected_index) else {
        return Vec::new();
    };
    if entry.kind == FileEntryKind::Directory {
        let path = listing.path.join(&entry.name);
        return navigate_file_manager(state, window_id, path);
    }
    state.status = format!("{} is a file · read-only browser", entry.name);
    Vec::new()
}

fn open_application(state: &mut AppState, application: ApplicationKind) -> u64 {
    let id = state.next_window_id;
    state.next_window_id += 1;
    state.current_workspace_mut().windows.push(Window {
        id,
        application,
        state: WindowState::Normal,
    });
    state.current_workspace_mut().focused_window = Some(id);
    if application == ApplicationKind::Terminal {
        state.set_terminal_status(id, TerminalStatus::Starting);
    }
    state.status = format!("Opened {}", application.title());
    id
}

fn close_focused_window(state: &mut AppState) -> Option<(u64, ApplicationKind)> {
    let focused = state.current_workspace().focused_window?;
    let workspace = state.current_workspace_mut();
    let application = workspace
        .windows
        .iter()
        .find(|window| window.id == focused)?
        .application;
    workspace.windows.retain(|window| window.id != focused);
    workspace.focused_window = workspace
        .windows
        .iter()
        .rev()
        .find(|window| window.state != WindowState::Minimized)
        .map(|window| window.id);
    state.status = "Window closed".to_owned();
    Some((focused, application))
}

fn focus_window(state: &mut AppState, id: u64) {
    if state
        .current_workspace()
        .windows
        .iter()
        .any(|window| window.id == id && window.state != WindowState::Minimized)
    {
        state.current_workspace_mut().focused_window = Some(id);
    }
}

fn focus_window_by_offset(state: &mut AppState, offset: isize) {
    let visible = state.visible_window_ids();
    if visible.is_empty() {
        state.current_workspace_mut().focused_window = None;
        return;
    }
    let current = state
        .current_workspace()
        .focused_window
        .and_then(|id| visible.iter().position(|item| *item == id))
        .unwrap_or(0);
    let next = (current as isize + offset).rem_euclid(visible.len() as isize) as usize;
    state.current_workspace_mut().focused_window = Some(visible[next]);
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn opening_and_closing_a_window_updates_workspace_state() {
        let mut state = AppState::default();
        reduce(
            &mut state,
            Action::OpenApplication(ApplicationKind::Terminal),
        );

        assert_eq!(state.current_workspace().windows.len(), 1);
        assert_eq!(
            state.focused_window().map(|window| window.application),
            Some(ApplicationKind::Terminal)
        );

        reduce(&mut state, Action::CloseWindow);
        assert!(state.current_workspace().windows.is_empty());
        assert_eq!(state.current_workspace().focused_window, None);
    }

    #[test]
    fn workspaces_keep_independent_windows() {
        let mut state = AppState::default();
        reduce(
            &mut state,
            Action::OpenApplication(ApplicationKind::Processes),
        );
        reduce(&mut state, Action::SwitchWorkspace(1));

        assert!(state.current_workspace().windows.is_empty());

        reduce(&mut state, Action::SwitchWorkspace(0));
        assert_eq!(state.current_workspace().windows.len(), 1);
    }

    #[test]
    fn refresh_returns_an_effect_and_enters_loading_state() {
        let mut state = AppState::default();
        let effects = reduce(&mut state, Action::Refresh);

        assert_eq!(state.status, "Refreshing local capabilities…");
        assert_eq!(state.system, Loadable::Loading);
        assert_eq!(effects, vec![Effect::RefreshCapabilities]);
    }

    #[test]
    fn opening_file_manager_schedules_directory_read() {
        let mut state = AppState::default();
        let effects = reduce(
            &mut state,
            Action::OpenApplication(ApplicationKind::FileManager),
        );

        assert_eq!(state.current_workspace().windows.len(), 1);
        let window_id = state.current_workspace().windows[0].id;
        assert!(state.file_manager(window_id).is_some());
        assert_eq!(effects.len(), 1);
        assert!(matches!(
            effects[0],
            Effect::ReadDirectory(id, _) if id == window_id
        ));
    }

    #[test]
    fn closing_file_manager_removes_per_window_state() {
        let mut state = AppState::default();
        reduce(
            &mut state,
            Action::OpenApplication(ApplicationKind::FileManager),
        );
        let window_id = state.current_workspace().windows[0].id;
        reduce(&mut state, Action::CloseWindow);
        assert!(state.file_manager(window_id).is_none());
    }

    #[test]
    fn parent_navigation_requests_parent_path() {
        let mut state = AppState::default();
        reduce(
            &mut state,
            Action::OpenApplication(ApplicationKind::FileManager),
        );
        let window_id = state.current_workspace().windows[0].id;
        state.file_managers.insert(
            window_id,
            crate::app::state::FileManagerState {
                current_path: PathBuf::from("/tmp/nested"),
                show_hidden: false,
                selected_index: 0,
                listing: Loadable::Ready(crate::machine::DirectoryListing {
                    path: PathBuf::from("/tmp/nested"),
                    entries: vec![],
                }),
            },
        );

        let effects = reduce(&mut state, Action::FileManagerParent);
        assert_eq!(effects.len(), 1);
        assert!(matches!(
            &effects[0],
            Effect::ReadDirectory(id, path) if *id == window_id && path == &PathBuf::from("/tmp")
        ));
    }

    #[test]
    fn move_file_selection_clamps_to_list_bounds() {
        let mut state = AppState::default();
        reduce(
            &mut state,
            Action::OpenApplication(ApplicationKind::FileManager),
        );
        let window_id = state.current_workspace().windows[0].id;
        state.file_managers.insert(
            window_id,
            crate::app::state::FileManagerState {
                current_path: PathBuf::from("/"),
                show_hidden: false,
                selected_index: 0,
                listing: Loadable::Ready(crate::machine::DirectoryListing {
                    path: PathBuf::from("/"),
                    entries: vec![
                        crate::machine::FileEntry {
                            name: "a".to_owned(),
                            kind: FileEntryKind::File,
                            size_bytes: None,
                            modified_secs: None,
                        },
                        crate::machine::FileEntry {
                            name: "b".to_owned(),
                            kind: FileEntryKind::File,
                            size_bytes: None,
                            modified_secs: None,
                        },
                    ],
                }),
            },
        );

        reduce(&mut state, Action::MoveFileSelection(5));
        assert_eq!(state.file_manager(window_id).unwrap().selected_index, 1);
        reduce(&mut state, Action::MoveFileSelection(-10));
        assert_eq!(state.file_manager(window_id).unwrap().selected_index, 0);
    }
}
