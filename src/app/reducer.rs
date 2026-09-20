use std::path::PathBuf;

use crate::{
    actions::Action,
    app::{
        AppState, ApplicationKind, FileManagerFocus, Loadable, TerminalStatus, Window, WindowState,
        file_manager::{
            DisplayRowKind, apply_sorted_listing, display_row_count, display_row_kind,
            ensure_selection_visible, shell_single_quoted,
        },
        process_manager::{matching_indices, selected_process},
    },
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
                    ApplicationKind::Processes => {
                        state.remove_process_manager(window_id);
                    }
                    _ => {}
                }
            }
        }
        Action::FocusWindowSlot(slot) => {
            focus_window_slot(state, slot);
            state.window_pick_mode = false;
        }
        Action::BeginWindowPick => {
            state.window_pick_mode = true;
            state.status = "Press 1–9 to focus a window · Esc to cancel".to_owned();
        }
        Action::CancelWindowPick => {
            state.window_pick_mode = false;
            state.status = "Window focus cancelled".to_owned();
        }
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
            {
                match manager.focus {
                    FileManagerFocus::Places if !manager.places.is_empty() => {
                        let next = manager.selected_place as i32 + offset;
                        manager.selected_place =
                            next.clamp(0, manager.places.len() as i32 - 1) as usize;
                    }
                    FileManagerFocus::List => {
                        if let Loadable::Ready(listing) = &manager.listing {
                            let count = display_row_count(listing, &manager.current_path);
                            if count > 0 {
                                let next = manager.selected_index as i32 + offset;
                                manager.selected_index = next.clamp(0, count as i32 - 1) as usize;
                                ensure_selection_visible(
                                    &mut manager.selected_index,
                                    &mut manager.scroll_offset,
                                    manager.visible_rows,
                                    count,
                                );
                            }
                        }
                    }
                    FileManagerFocus::Places => {}
                }
            }
        }
        Action::OpenSelectedEntry => return open_selected_entry(state),
        Action::FileManagerGoUp => return file_manager_go_up(state),
        Action::FileManagerGoHome => {
            if let Some(window_id) = focused_file_manager_window(state) {
                let path = default_start_path();
                return navigate_file_manager(state, window_id, path, true);
            }
        }
        Action::FileManagerGoBack => {
            if let Some(window_id) = focused_file_manager_window(state)
                && let Some(manager) = state.file_manager_mut(window_id)
                && manager.history_index > 0
            {
                manager.history_index -= 1;
                let path = manager.history[manager.history_index].clone();
                return navigate_file_manager(state, window_id, path, false);
            }
        }
        Action::FileManagerTogglePane => {
            if let Some(window_id) = focused_file_manager_window(state)
                && let Some(manager) = state.file_manager_mut(window_id)
            {
                manager.focus = match manager.focus {
                    FileManagerFocus::Places => FileManagerFocus::List,
                    FileManagerFocus::List => FileManagerFocus::Places,
                };
            }
        }
        Action::FileManagerPageScroll(pages) => {
            if let Some(window_id) = focused_file_manager_window(state)
                && let Some(manager) = state.file_manager_mut(window_id)
                && let Loadable::Ready(listing) = &manager.listing
            {
                let count = display_row_count(listing, &manager.current_path);
                let delta = pages * manager.visible_rows.max(1) as i32;
                let next = manager.selected_index as i32 + delta;
                manager.selected_index = next.clamp(0, count.saturating_sub(1) as i32) as usize;
                ensure_selection_visible(
                    &mut manager.selected_index,
                    &mut manager.scroll_offset,
                    manager.visible_rows,
                    count,
                );
            }
        }
        Action::FileManagerSetSort(column) => {
            if let Some(window_id) = focused_file_manager_window(state)
                && let Some(manager) = state.file_manager_mut(window_id)
            {
                if manager.sort.column == column {
                    manager.sort.ascending = !manager.sort.ascending;
                } else {
                    manager.sort.column = column;
                    manager.sort.ascending = true;
                }
                if let Loadable::Ready(listing) = &manager.listing {
                    let sorted = apply_sorted_listing(listing.clone(), manager.sort);
                    manager.listing = Loadable::Ready(sorted);
                }
                manager.clamp_selection();
            }
        }
        Action::ToggleFileManagerHidden => {
            if let Some(window_id) = focused_file_manager_window(state)
                && let Some(manager) = state.file_manager_mut(window_id)
            {
                manager.show_hidden = !manager.show_hidden;
                let path = manager.current_path.clone();
                return navigate_file_manager(state, window_id, path, false);
            }
        }
        Action::ReloadFileManager => return reload_focused_file_manager(state),
        Action::SelectFileManagerRow(window_id, index) => {
            if focused_file_manager_window(state) == Some(window_id)
                && let Some(manager) = state.file_manager_mut(window_id)
            {
                manager.focus = FileManagerFocus::List;
                if let Loadable::Ready(listing) = &manager.listing {
                    let count = display_row_count(listing, &manager.current_path);
                    if index < count {
                        manager.selected_index = index;
                        ensure_selection_visible(
                            &mut manager.selected_index,
                            &mut manager.scroll_offset,
                            manager.visible_rows,
                            count,
                        );
                    }
                }
            }
        }
        Action::SelectFileManagerPlace(window_id, index) => {
            if focused_file_manager_window(state) == Some(window_id)
                && let Some(manager) = state.file_manager_mut(window_id)
                && index < manager.places.len()
            {
                manager.focus = FileManagerFocus::Places;
                manager.selected_place = index;
            }
        }
        Action::ToggleInputDebug => {
            state.input_debug = !state.input_debug;
            state.input_debug_line = if state.input_debug {
                "Input debug on (Ctrl+Alt+D off)".to_owned()
            } else {
                String::new()
            };
        }
        Action::MoveProcessSelection(offset) => {
            if let Some(window_id) = focused_process_window(state) {
                let count = process_match_count(state, window_id);
                if let Some(manager) = state.process_manager_mut(window_id)
                    && count > 0
                {
                    let next = manager.selected_index as i32 + offset;
                    manager.selected_index = next.clamp(0, count as i32 - 1) as usize;
                    manager.clamp_selection(count);
                }
            }
        }
        Action::ProcessPageScroll(pages) => {
            if let Some(window_id) = focused_process_window(state) {
                let count = process_match_count(state, window_id);
                if let Some(manager) = state.process_manager_mut(window_id) {
                    let delta = pages * manager.visible_rows.max(1) as i32;
                    let next = manager.selected_index as i32 + delta;
                    manager.selected_index = next.clamp(0, count.saturating_sub(1) as i32) as usize;
                    manager.clamp_selection(count);
                }
            }
        }
        Action::ProcessFilterBegin => {
            if let Some(window_id) = focused_process_window(state)
                && let Some(manager) = state.process_manager_mut(window_id)
            {
                manager.filter_active = true;
                manager.filter.clear();
                manager.selected_index = 0;
                manager.scroll_offset = 0;
                state.status = "Process filter · type to search · Esc when done".to_owned();
            }
        }
        Action::ProcessFilterPush(character) => {
            if let Some(window_id) = focused_process_window(state) {
                if let Some(manager) = state.process_manager_mut(window_id) {
                    manager.filter.push(character);
                }
                let count = process_match_count(state, window_id);
                if let Some(manager) = state.process_manager_mut(window_id) {
                    manager.clamp_selection(count);
                }
            }
        }
        Action::ProcessFilterBackspace => {
            if let Some(window_id) = focused_process_window(state) {
                if let Some(manager) = state.process_manager_mut(window_id) {
                    manager.filter.pop();
                }
                let count = process_match_count(state, window_id);
                if let Some(manager) = state.process_manager_mut(window_id) {
                    manager.clamp_selection(count);
                }
            }
        }
        Action::ProcessFilterEnd => {
            if let Some(window_id) = focused_process_window(state)
                && let Some(manager) = state.process_manager_mut(window_id)
            {
                manager.filter_active = false;
                state.status = "Process filter applied".to_owned();
            }
        }
        Action::ProcessKillSelected => {
            if let Loadable::Ready(processes) = &state.processes {
                if let Some(window_id) = focused_process_window(state)
                    && let Some(manager) = state.process_manager(window_id)
                    && let Some(process) =
                        selected_process(processes, &manager.filter, manager.selected_index)
                {
                    state.status = format!("Sending SIGTERM to {} ({})", process.name, process.pid);
                    return vec![Effect::KillProcess(process.pid)];
                }
            }
            state.status = "No process selected".to_owned();
        }
        Action::SelectProcessRow(window_id, index) => {
            if focused_process_window(state) == Some(window_id) {
                let count = process_match_count(state, window_id);
                if let Some(manager) = state.process_manager_mut(window_id)
                    && index < count
                {
                    manager.selected_index = index;
                    manager.clamp_selection(count);
                }
            }
        }
        Action::FileManagerOpenInTerminal => return open_selection_in_terminal(state),
    }
    Vec::new()
}

fn focused_process_window(state: &AppState) -> Option<u64> {
    state
        .focused_window()
        .filter(|window| window.application == ApplicationKind::Processes)
        .map(|window| window.id)
}

fn process_match_count(state: &AppState, window_id: u64) -> usize {
    let filter = state
        .process_manager(window_id)
        .map(|manager| manager.filter.clone())
        .unwrap_or_default();
    match &state.processes {
        Loadable::Ready(processes) => matching_indices(processes, &filter).len(),
        _ => 0,
    }
}

fn open_selection_in_terminal(state: &mut AppState) -> Vec<Effect> {
    let Some(file_window_id) = focused_file_manager_window(state) else {
        return Vec::new();
    };
    let Some(manager) = state.file_manager(file_window_id) else {
        return Vec::new();
    };
    let command = if manager.focus == FileManagerFocus::Places {
        manager
            .places
            .get(manager.selected_place)
            .map(|place| format!("cd {}\n", shell_single_quoted(&place.path)))
    } else {
        let Loadable::Ready(listing) = &manager.listing else {
            state.status = "Directory is still loading".to_owned();
            return Vec::new();
        };
        match display_row_kind(&manager.current_path, listing, manager.selected_index) {
            Some(DisplayRowKind::Parent) => {
                let parent = manager.current_path.parent().map(|path| path.to_path_buf());
                parent.map(|path| format!("cd {}\n", shell_single_quoted(&path)))
            }
            Some(DisplayRowKind::Entry) => {
                let Some(entry) = crate::app::file_manager::display_entry(
                    &manager.current_path,
                    listing,
                    manager.selected_index,
                ) else {
                    return Vec::new();
                };
                let path = listing.path.join(&entry.name);
                if entry.kind == FileEntryKind::Directory {
                    Some(format!("cd {}\n", shell_single_quoted(&path)))
                } else {
                    let parent = shell_single_quoted(listing.path.as_path());
                    let file = shell_single_quoted(&path);
                    Some(format!(
                        "cd {} && (command -v less >/dev/null && less {file} || cat {file})\n",
                        parent
                    ))
                }
            }
            None => None,
        }
    };
    let Some(command) = command else {
        state.status = "Nothing to open in a terminal".to_owned();
        return Vec::new();
    };
    let terminal_id = open_application(state, ApplicationKind::Terminal);
    state.status = "Opened terminal for selection".to_owned();
    vec![
        Effect::StartTerminal(terminal_id),
        Effect::WriteTerminal(terminal_id, command.into_bytes()),
    ]
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
        ApplicationKind::Processes => {
            state.init_process_manager(window_id);
            Vec::new()
        }
        _ => Vec::new(),
    }
}

fn navigate_file_manager(
    state: &mut AppState,
    window_id: u64,
    path: PathBuf,
    push_history: bool,
) -> Vec<Effect> {
    if let Some(manager) = state.file_manager_mut(window_id) {
        manager.current_path = path.clone();
        manager.listing = Loadable::Loading;
        manager.selected_index = 0;
        manager.scroll_offset = 0;
        if push_history {
            push_file_manager_history(manager, path.clone());
        }
        vec![Effect::ReadDirectory(window_id, path)]
    } else {
        Vec::new()
    }
}

fn push_file_manager_history(manager: &mut crate::app::state::FileManagerState, path: PathBuf) {
    if manager.history.get(manager.history_index) == Some(&path) {
        return;
    }
    manager.history.truncate(manager.history_index + 1);
    manager.history.push(path);
    manager.history_index = manager.history.len() - 1;
}

fn file_manager_go_up(state: &mut AppState) -> Vec<Effect> {
    if let Some(window_id) = focused_file_manager_window(state)
        && let Some(manager) = state.file_manager(window_id)
        && let Some(parent) = manager.current_path.parent()
        && parent != manager.current_path.as_path()
    {
        return navigate_file_manager(state, window_id, parent.to_path_buf(), true);
    }
    state.status = "Already at the root directory".to_owned();
    Vec::new()
}

fn reload_focused_file_manager(state: &mut AppState) -> Vec<Effect> {
    if let Some(window_id) = focused_file_manager_window(state)
        && let Some(manager) = state.file_manager(window_id)
    {
        let path = manager.current_path.clone();
        state.status = format!("Reading {}", path.display());
        return navigate_file_manager(state, window_id, path, false);
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
    if manager.focus == FileManagerFocus::Places {
        if let Some(place) = manager.places.get(manager.selected_place) {
            let path = place.path.clone();
            return navigate_file_manager(state, window_id, path, true);
        }
        return Vec::new();
    }
    let Loadable::Ready(listing) = &manager.listing else {
        return Vec::new();
    };
    match display_row_kind(&manager.current_path, listing, manager.selected_index) {
        Some(DisplayRowKind::Parent) => return file_manager_go_up(state),
        Some(DisplayRowKind::Entry) => {
            let Some(entry) = crate::app::file_manager::display_entry(
                &manager.current_path,
                listing,
                manager.selected_index,
            ) else {
                return Vec::new();
            };
            if entry.kind == FileEntryKind::Directory {
                let path = listing.path.join(&entry.name);
                return navigate_file_manager(state, window_id, path, true);
            }
            state.status = format!("{} · press o to open in a terminal", entry.name);
        }
        None => {}
    }
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
    let workspace = state.current_workspace_mut();
    if let Some(window) = workspace.windows.iter_mut().find(|window| window.id == id) {
        if window.state == WindowState::Minimized {
            window.state = WindowState::Normal;
        }
        workspace.focused_window = Some(id);
    }
}

fn focus_window_slot(state: &mut AppState, slot: u8) {
    if !(1..=9).contains(&slot) {
        return;
    }
    let windows = state
        .current_workspace()
        .windows
        .iter()
        .map(|window| window.id)
        .collect::<Vec<_>>();
    let index = (slot - 1) as usize;
    if let Some(id) = windows.get(index) {
        focus_window(state, *id);
        state.status = format!("Window {slot}");
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
        let mut manager = crate::app::state::FileManagerState::new(PathBuf::from("/tmp/nested"));
        manager.listing = Loadable::Ready(crate::machine::DirectoryListing {
            path: PathBuf::from("/tmp/nested"),
            entries: vec![],
        });
        state.file_managers.insert(window_id, manager);

        let effects = reduce(&mut state, Action::FileManagerGoUp);
        assert_eq!(effects.len(), 1);
        assert!(matches!(
            &effects[0],
            Effect::ReadDirectory(id, path) if *id == window_id && path == &PathBuf::from("/tmp")
        ));
    }

    #[test]
    fn focusing_a_minimized_window_restores_it() {
        let mut state = AppState::default();
        reduce(
            &mut state,
            Action::OpenApplication(ApplicationKind::Terminal),
        );
        reduce(
            &mut state,
            Action::OpenApplication(ApplicationKind::FileManager),
        );
        let minimized_id = state.current_workspace().windows[0].id;
        reduce(&mut state, Action::FocusWindowSlot(1));
        reduce(&mut state, Action::MinimizeWindow);
        assert_eq!(
            state
                .current_workspace()
                .windows
                .iter()
                .find(|window| window.id == minimized_id)
                .map(|window| window.state),
            Some(WindowState::Minimized)
        );
        reduce(&mut state, Action::FocusWindow(minimized_id));
        assert_eq!(
            state
                .current_workspace()
                .windows
                .iter()
                .find(|window| window.id == minimized_id)
                .map(|window| window.state),
            Some(WindowState::Normal)
        );
        assert_eq!(state.current_workspace().focused_window, Some(minimized_id));
    }

    #[test]
    fn move_file_selection_clamps_to_list_bounds() {
        let mut state = AppState::default();
        reduce(
            &mut state,
            Action::OpenApplication(ApplicationKind::FileManager),
        );
        let window_id = state.current_workspace().windows[0].id;
        let mut manager = crate::app::state::FileManagerState::new(PathBuf::from("/"));
        manager.visible_rows = 20;
        manager.listing = Loadable::Ready(crate::machine::DirectoryListing {
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
        });
        state.file_managers.insert(window_id, manager);

        reduce(&mut state, Action::MoveFileSelection(5));
        assert_eq!(state.file_manager(window_id).unwrap().selected_index, 1);
        reduce(&mut state, Action::MoveFileSelection(-10));
        assert_eq!(state.file_manager(window_id).unwrap().selected_index, 0);
    }
}
