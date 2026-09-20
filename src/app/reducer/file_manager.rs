use std::path::PathBuf;

use crate::{
    actions::FileManagerAction,
    app::{
        AppState, ApplicationKind, FileManagerFocus, Loadable,
        file_manager::{
            CreateKind, DisplayRowKind, apply_sorted_listing, display_row_count, display_row_kind,
            ensure_selection_visible, selected_entry_path, shell_single_quoted,
            validate_rename_input,
        },
        state::FileManagerDialog,
    },
    machine::{FileEntryKind, local::default_start_path},
};

use super::{Effect, shell};

pub(super) fn reduce(state: &mut AppState, action: FileManagerAction) -> Vec<Effect> {
    match action {
        FileManagerAction::MoveFileSelection(offset) => {
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
        FileManagerAction::OpenSelectedEntry => return open_selected_entry(state),
        FileManagerAction::FileManagerGoUp => return file_manager_go_up(state),
        FileManagerAction::FileManagerGoHome => {
            if let Some(window_id) = focused_file_manager_window(state) {
                let path = default_start_path();
                return navigate_file_manager(state, window_id, path, true);
            }
        }
        FileManagerAction::FileManagerGoBack => {
            if let Some(window_id) = focused_file_manager_window(state)
                && let Some(manager) = state.file_manager_mut(window_id)
                && manager.history_index > 0
            {
                manager.history_index -= 1;
                let path = manager.history[manager.history_index].clone();
                return navigate_file_manager(state, window_id, path, false);
            }
        }
        FileManagerAction::FileManagerTogglePane => {
            if let Some(window_id) = focused_file_manager_window(state)
                && let Some(manager) = state.file_manager_mut(window_id)
            {
                manager.focus = match manager.focus {
                    FileManagerFocus::Places => FileManagerFocus::List,
                    FileManagerFocus::List => FileManagerFocus::Places,
                };
            }
        }
        FileManagerAction::FileManagerPageScroll(pages) => {
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
        FileManagerAction::FileManagerSetSort(column) => {
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
        FileManagerAction::ToggleFileManagerHidden => {
            if let Some(window_id) = resolve_file_manager_window(state)
                && let Some(manager) = state.file_manager_mut(window_id)
            {
                manager.show_hidden = !manager.show_hidden;
                let path = manager.current_path.clone();
                return navigate_file_manager(state, window_id, path, false);
            }
        }
        FileManagerAction::ReloadFileManager => return reload_focused_file_manager(state),
        FileManagerAction::SelectFileManagerRow(window_id, index) => {
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
        FileManagerAction::SelectFileManagerPlace(window_id, index) => {
            if focused_file_manager_window(state) == Some(window_id)
                && let Some(manager) = state.file_manager_mut(window_id)
                && index < manager.places.len()
            {
                manager.focus = FileManagerFocus::Places;
                manager.selected_place = index;
            }
        }
        FileManagerAction::FileManagerOpenInTerminal => return open_selection_in_terminal(state),
        FileManagerAction::FileManagerBeginGoToPath => {
            if let Some(window_id) = resolve_file_manager_window(state)
                && let Some(manager) = state.file_manager_mut(window_id)
            {
                manager.dialog = FileManagerDialog::GoToPath {
                    input: manager.current_path.display().to_string(),
                };
                state.status = "Go to path · type folder · Enter · Esc cancel".to_owned();
            } else {
                state.status = "Open a file manager window first".to_owned();
            }
        }
        FileManagerAction::FileManagerOpenInViewer => {
            if let Some(window_id) = resolve_file_manager_window(state)
                && let Some(manager) = state.file_manager(window_id)
                && let Some(path) = selected_entry_path(manager)
            {
                return super::text_viewer::open_path_effects(state, path);
            }
            state.status = "Select a file to view".to_owned();
        }
        FileManagerAction::FileManagerOpenWithSystem => {
            if let Some(window_id) = resolve_file_manager_window(state)
                && let Some(manager) = state.file_manager(window_id)
                && let Some(path) = selected_entry_path(manager)
            {
                return vec![Effect::FileManager(
                    crate::app::effects::FileManagerEffect::OpenWithSystem(path),
                )];
            }
            state.status = "Select a file to open with the system handler".to_owned();
        }
        FileManagerAction::FileManagerRequestDelete => {
            if let Some(window_id) = resolve_file_manager_window(state)
                && let Some(manager) = state.file_manager(window_id)
            {
                if let Some(path) = selected_entry_path(manager) {
                    let label = path
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_else(|| path.display().to_string());
                    state.status =
                        format!("Move to trash / delete {label}?  y confirm · n/Esc cancel");
                    if let Some(manager) = state.file_manager_mut(window_id) {
                        manager.dialog = FileManagerDialog::DeleteConfirm { path, label };
                    }
                } else {
                    state.status = "Select a file or folder to delete (not ..)".to_owned();
                }
            } else {
                state.status = "Open a file manager window first".to_owned();
            }
        }
        FileManagerAction::FileManagerConfirmDelete => {
            if let Some(window_id) = resolve_file_manager_window(state)
                && let Some(manager) = state.file_manager(window_id)
                && let FileManagerDialog::DeleteConfirm { path, .. } = &manager.dialog
            {
                let path = path.clone();
                if let Some(manager) = state.file_manager_mut(window_id) {
                    manager.dialog = FileManagerDialog::None;
                }
                state.status = format!("Removing {}…", path.display());
                return vec![Effect::FileManager(
                    crate::app::effects::FileManagerEffect::DeletePath(window_id, path),
                )];
            }
        }
        FileManagerAction::FileManagerCancelDialog => {
            if let Some(window_id) = resolve_file_manager_window(state)
                && let Some(manager) = state.file_manager_mut(window_id)
            {
                manager.dialog = FileManagerDialog::None;
                state.status = "Cancelled".to_owned();
            }
        }
        FileManagerAction::FileManagerBeginRename => {
            if let Some(window_id) = resolve_file_manager_window(state)
                && let Some(manager) = state.file_manager(window_id)
                && let Some(path) = selected_entry_path(manager)
            {
                let input = path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default();
                if let Some(manager) = state.file_manager_mut(window_id) {
                    manager.dialog = FileManagerDialog::Rename { path, input };
                    state.status = "Rename · type new name · Enter save · Esc cancel".to_owned();
                }
            } else if crate::app::file_manager::target_file_manager_window(state).is_none() {
                state.status = "Open a file manager window first".to_owned();
            } else {
                state.status = "Select a file or folder to rename (not ..)".to_owned();
            }
        }
        FileManagerAction::FileManagerBeginCreate(kind) => {
            if let Some(window_id) = resolve_file_manager_window(state) {
                if let Some(manager) = state.file_manager_mut(window_id) {
                    manager.dialog = FileManagerDialog::Create {
                        kind,
                        input: String::new(),
                    };
                    let label = match kind {
                        CreateKind::File => "file",
                        CreateKind::Directory => "folder",
                    };
                    state.status = format!("New {label} · type name · Enter create · Esc cancel");
                }
            } else {
                state.status = "Open a file manager window first".to_owned();
            }
        }
        FileManagerAction::FileManagerDialogPush(character) => {
            if let Some(window_id) = resolve_file_manager_window(state)
                && let Some(manager) = state.file_manager_mut(window_id)
            {
                match &mut manager.dialog {
                    FileManagerDialog::Rename { input, .. }
                    | FileManagerDialog::Create { input, .. }
                    | FileManagerDialog::GoToPath { input } => input.push(character),
                    _ => {}
                }
            }
        }
        FileManagerAction::FileManagerDialogBackspace => {
            if let Some(window_id) = resolve_file_manager_window(state)
                && let Some(manager) = state.file_manager_mut(window_id)
            {
                match &mut manager.dialog {
                    FileManagerDialog::Rename { input, .. }
                    | FileManagerDialog::Create { input, .. }
                    | FileManagerDialog::GoToPath { input } => {
                        input.pop();
                    }
                    _ => {}
                }
            }
        }
        FileManagerAction::FileManagerDialogCommit => return commit_file_manager_dialog(state),
    }
    Vec::new()
}

fn commit_file_manager_dialog(state: &mut AppState) -> Vec<Effect> {
    let Some(window_id) = resolve_file_manager_window(state) else {
        return Vec::new();
    };
    let (dialog, current_path) = {
        let Some(manager) = state.file_manager(window_id) else {
            return Vec::new();
        };
        (manager.dialog.clone(), manager.current_path.clone())
    };
    match dialog {
        FileManagerDialog::Rename { path, input } => {
            if let Err(reason) = validate_rename_input(&input) {
                state.status = format!("Rename: {reason}");
                return Vec::new();
            }
            let name = input.trim();
            let to = path
                .parent()
                .map(|parent| parent.join(name))
                .unwrap_or_else(|| PathBuf::from(name));
            if to == path {
                if let Some(manager) = state.file_manager_mut(window_id) {
                    manager.dialog = FileManagerDialog::None;
                }
                state.status = "Name unchanged".to_owned();
                return Vec::new();
            }
            if let Some(manager) = state.file_manager_mut(window_id) {
                manager.dialog = FileManagerDialog::None;
            }
            vec![Effect::FileManager(
                crate::app::effects::FileManagerEffect::RenamePath(window_id, path, to),
            )]
        }
        FileManagerDialog::Create { kind, input } => {
            if let Err(reason) = validate_rename_input(&input) {
                state.status = format!("Create: {reason}");
                return Vec::new();
            }
            let name = input.trim();
            let path = current_path.join(name);
            if let Some(manager) = state.file_manager_mut(window_id) {
                manager.dialog = FileManagerDialog::None;
            }
            vec![Effect::FileManager(
                crate::app::effects::FileManagerEffect::CreateEntry(window_id, path, kind),
            )]
        }
        FileManagerDialog::GoToPath { input } => {
            let path = PathBuf::from(input.trim());
            if path.as_os_str().is_empty() {
                state.status = "Enter a path".to_owned();
                return Vec::new();
            }
            if let Some(manager) = state.file_manager_mut(window_id) {
                manager.dialog = FileManagerDialog::None;
            }
            navigate_file_manager(state, window_id, path, true)
        }
        _ => Vec::new(),
    }
}

fn open_selection_in_terminal(state: &mut AppState) -> Vec<Effect> {
    let Some(file_window_id) = resolve_file_manager_window(state) else {
        state.status = "Open a file manager window first".to_owned();
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
                    state.status = format!(
                        "{} · o opens a terminal in folders — e view · Shift+O system",
                        entry.name
                    );
                    return Vec::new();
                }
            }
            None => None,
        }
    };
    let Some(command) = command else {
        state.status = "Nothing to open in a terminal".to_owned();
        return Vec::new();
    };
    let terminal_id = shell::open_application(state, ApplicationKind::Terminal);
    state.status = "Opened terminal for selection".to_owned();
    vec![
        Effect::Terminal(crate::app::effects::TerminalEffect::Start(terminal_id)),
        Effect::Terminal(crate::app::effects::TerminalEffect::Write(
            terminal_id,
            command.into_bytes(),
        )),
    ]
}

fn focused_file_manager_window(state: &AppState) -> Option<u64> {
    state
        .focused_window()
        .filter(|window| window.application == ApplicationKind::FileManager)
        .map(|window| window.id)
}

fn resolve_file_manager_window(state: &mut AppState) -> Option<u64> {
    if let Some(window_id) = focused_file_manager_window(state) {
        return Some(window_id);
    }
    let window_id = crate::app::file_manager::target_file_manager_window(state)?;
    shell::focus_window(state, window_id);
    Some(window_id)
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
        vec![Effect::FileManager(
            crate::app::effects::FileManagerEffect::ReadDirectory(window_id, path),
        )]
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
    if let Some(window_id) = resolve_file_manager_window(state)
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
            state.status = format!("{} · e to view · Shift+O system", entry.name);
        }
        None => {}
    }
    Vec::new()
}
