use std::path::PathBuf;

use crate::{
    actions::{Action, FileManagerAction, PaletteAction, ShellAction},
    app::{AppState, ApplicationKind, Loadable, WindowState, state::FileManagerDialog},
    machine::FileEntryKind,
};

use crate::app::effects::{FileManagerEffect, TerminalEffect};

use super::{Effect, reduce};

#[test]
fn opening_and_closing_a_window_updates_workspace_state() {
    let mut state = AppState::default();
    reduce(
        &mut state,
        Action::Shell(ShellAction::OpenApplication(ApplicationKind::Terminal)),
    );

    assert_eq!(state.current_workspace().windows.len(), 1);
    assert_eq!(
        state.focused_window().map(|window| window.application),
        Some(ApplicationKind::Terminal)
    );

    reduce(&mut state, Action::Shell(ShellAction::CloseWindow));
    assert!(state.current_workspace().windows.is_empty());
    assert_eq!(state.current_workspace().focused_window, None);
}

#[test]
fn workspaces_keep_independent_windows() {
    let mut state = AppState::default();
    reduce(
        &mut state,
        Action::Shell(ShellAction::OpenApplication(ApplicationKind::Processes)),
    );
    reduce(&mut state, Action::Shell(ShellAction::SwitchWorkspace(1)));

    assert!(state.current_workspace().windows.is_empty());

    reduce(&mut state, Action::Shell(ShellAction::SwitchWorkspace(0)));
    assert_eq!(state.current_workspace().windows.len(), 1);
}

#[test]
fn refresh_returns_an_effect_and_enters_loading_state() {
    let mut state = AppState::default();
    let effects = reduce(&mut state, Action::Shell(ShellAction::Refresh));

    assert_eq!(state.status, "Refreshing local capabilities…");
    assert_eq!(
        state.system_for(&crate::machine::MachineId::Local),
        Loadable::Loading
    );
    assert_eq!(effects, vec![Effect::RefreshCapabilities]);
}

#[test]
fn opening_file_manager_schedules_directory_read() {
    let mut state = AppState::default();
    let effects = reduce(
        &mut state,
        Action::Shell(ShellAction::OpenApplication(ApplicationKind::FileManager)),
    );

    assert_eq!(state.current_workspace().windows.len(), 1);
    let window_id = state.current_workspace().windows[0].id;
    assert!(state.file_manager(window_id).is_some());
    assert_eq!(effects.len(), 1);
    assert!(matches!(
        effects[0],
        Effect::FileManager(FileManagerEffect::ReadDirectory(id, _)) if id == window_id
    ));
}

#[test]
fn closing_file_manager_removes_per_window_state() {
    let mut state = AppState::default();
    reduce(
        &mut state,
        Action::Shell(ShellAction::OpenApplication(ApplicationKind::FileManager)),
    );
    let window_id = state.current_workspace().windows[0].id;
    reduce(&mut state, Action::Shell(ShellAction::CloseWindow));
    assert!(state.file_manager(window_id).is_none());
}

#[test]
fn parent_navigation_requests_parent_path() {
    let mut state = AppState::default();
    reduce(
        &mut state,
        Action::Shell(ShellAction::OpenApplication(ApplicationKind::FileManager)),
    );
    let window_id = state.current_workspace().windows[0].id;
    let mut manager = crate::app::state::FileManagerState::new(PathBuf::from("/tmp/nested"));
    manager.listing = Loadable::Ready(crate::machine::DirectoryListing {
        path: PathBuf::from("/tmp/nested"),
        entries: vec![],
    });
    state.file_managers.insert(window_id, manager);

    let effects = reduce(
        &mut state,
        Action::FileManager(FileManagerAction::FileManagerGoUp),
    );
    assert_eq!(effects.len(), 1);
    assert!(matches!(
        &effects[0],
        Effect::FileManager(FileManagerEffect::ReadDirectory(id, path))
            if *id == window_id && path == &PathBuf::from("/tmp")
    ));
}

#[test]
fn palette_bang_prefix_opens_terminal_with_command() {
    let mut state = AppState::default();
    reduce(&mut state, Action::Palette(PaletteAction::ToggleLauncher));
    for character in "!echo hi".chars() {
        reduce(
            &mut state,
            Action::Palette(PaletteAction::PaletteQueryPush(character)),
        );
    }
    let effects = reduce(
        &mut state,
        Action::Palette(PaletteAction::ExecuteLauncherSelection),
    );
    assert!(!state.launcher_open);
    assert_eq!(
        state.focused_window().map(|window| window.application),
        Some(ApplicationKind::Terminal)
    );
    assert_eq!(effects.len(), 2);
    assert!(matches!(
        effects[0],
        Effect::Terminal(TerminalEffect::Start(_))
    ));
    assert!(matches!(
        effects[1],
        Effect::Terminal(TerminalEffect::Write(_, _))
    ));
}

#[test]
fn palette_filter_narrows_and_opens_terminal() {
    let mut state = AppState::default();
    reduce(&mut state, Action::Palette(PaletteAction::ToggleLauncher));
    reduce(
        &mut state,
        Action::Palette(PaletteAction::PaletteQueryPush('t')),
    );
    reduce(
        &mut state,
        Action::Palette(PaletteAction::PaletteQueryPush('e')),
    );
    reduce(
        &mut state,
        Action::Palette(PaletteAction::PaletteQueryPush('r')),
    );
    reduce(
        &mut state,
        Action::Palette(PaletteAction::PaletteQueryPush('m')),
    );

    let filtered = crate::app::palette::filtered_entries(&state);
    assert!(filtered.iter().any(|entry| matches!(
        entry.action,
        Action::Shell(ShellAction::OpenApplication(ApplicationKind::Terminal))
    )));
    reduce(
        &mut state,
        Action::Palette(PaletteAction::ExecuteLauncherSelection),
    );
    assert!(!state.launcher_open);
    assert_eq!(
        state.focused_window().map(|window| window.application),
        Some(ApplicationKind::Terminal)
    );
}

#[test]
fn focusing_a_minimized_window_restores_it() {
    let mut state = AppState::default();
    reduce(
        &mut state,
        Action::Shell(ShellAction::OpenApplication(ApplicationKind::Terminal)),
    );
    reduce(
        &mut state,
        Action::Shell(ShellAction::OpenApplication(ApplicationKind::FileManager)),
    );
    let minimized_id = state.current_workspace().windows[0].id;
    reduce(&mut state, Action::Shell(ShellAction::FocusWindowSlot(1)));
    reduce(&mut state, Action::Shell(ShellAction::MinimizeWindow));
    assert_eq!(
        state
            .current_workspace()
            .windows
            .iter()
            .find(|window| window.id == minimized_id)
            .map(|window| window.state),
        Some(WindowState::Minimized)
    );
    reduce(
        &mut state,
        Action::Shell(ShellAction::FocusWindow(minimized_id)),
    );
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
fn rename_dialog_commit_schedules_rename_effect() {
    let mut state = AppState::default();
    reduce(
        &mut state,
        Action::Shell(ShellAction::OpenApplication(ApplicationKind::FileManager)),
    );
    let window_id = state.current_workspace().windows[0].id;
    let mut manager = crate::app::state::FileManagerState::new(PathBuf::from("/tmp"));
    manager.listing = Loadable::Ready(crate::machine::DirectoryListing {
        path: PathBuf::from("/tmp"),
        entries: vec![crate::machine::FileEntry {
            name: "old.txt".to_owned(),
            kind: FileEntryKind::File,
            size_bytes: Some(0),
            modified_secs: None,
        }],
    });
    manager.selected_index = 1;
    manager.dialog = FileManagerDialog::Rename {
        path: PathBuf::from("/tmp/old.txt"),
        input: "new.txt".to_owned(),
    };
    state.file_managers.insert(window_id, manager);

    let effects = reduce(
        &mut state,
        Action::FileManager(FileManagerAction::FileManagerDialogCommit),
    );
    assert_eq!(effects.len(), 1);
    assert!(matches!(
        &effects[0],
        Effect::FileManager(FileManagerEffect::RenamePath(id, from, to))
            if *id == window_id
                && from == &PathBuf::from("/tmp/old.txt")
                && to == &PathBuf::from("/tmp/new.txt")
    ));
}

#[test]
fn move_file_selection_clamps_to_list_bounds() {
    let mut state = AppState::default();
    reduce(
        &mut state,
        Action::Shell(ShellAction::OpenApplication(ApplicationKind::FileManager)),
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

    reduce(
        &mut state,
        Action::FileManager(FileManagerAction::MoveFileSelection(5)),
    );
    assert_eq!(state.file_manager(window_id).unwrap().selected_index, 1);
    reduce(
        &mut state,
        Action::FileManager(FileManagerAction::MoveFileSelection(-10)),
    );
    assert_eq!(state.file_manager(window_id).unwrap().selected_index, 0);
}
