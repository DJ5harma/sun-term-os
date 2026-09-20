use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Frame, layout::Rect};

use crate::{
    actions::{Action, FileManagerAction},
    app::{
        SortColumn,
        effects::{Effect, FileManagerEffect},
        file_manager::CreateKind,
        palette::PaletteEntry,
        state::{AppState, FileManagerDialog},
    },
    domain::{ApplicationKind, Window, WindowId},
    machine::local::default_start_path,
    ui::interaction::InteractionMap,
};

use super::{AppKeyResult, BuiltInApp, shell_keys::quick_launch_action};

pub const APP: BuiltInApp = BuiltInApp {
    kind: ApplicationKind::FileManager,
    title: "File Manager",
    short_title: "Files",
    description: "Browse files and folders",
    quick_launch_key: Some('f'),
    on_open,
    on_close,
    on_open_window: None,
    render,
    dispatch_key,
    desktop_scroll: Some(|delta| {
        Action::FileManager(FileManagerAction::FileManagerPageScroll(delta))
    }),
    palette_extras: Some(palette_entries),
};

pub fn on_open(state: &mut AppState, window_id: WindowId) -> Vec<Effect> {
    let path = default_start_path();
    state.init_file_manager(window_id, path.clone());
    vec![Effect::FileManager(FileManagerEffect::ReadDirectory(
        window_id, path,
    ))]
}

pub fn on_close(state: &mut AppState, window_id: WindowId) -> Vec<Effect> {
    state.remove_file_manager(window_id);
    Vec::new()
}

fn file_manager_rename_shortcut(key: KeyEvent) -> bool {
    if key.modifiers.intersects(KeyModifiers::CONTROL) {
        return false;
    }
    match key.code {
        KeyCode::Char('R') => true,
        KeyCode::Char('r') => key.modifiers.intersects(KeyModifiers::SHIFT),
        _ => false,
    }
}

pub fn dispatch_key(key: KeyEvent, state: &AppState) -> AppKeyResult {
    let manager = state
        .focused_window()
        .filter(|window| window.application == ApplicationKind::FileManager)
        .and_then(|window| state.file_manager(window.id))
        .or_else(|| {
            crate::app::file_manager::target_file_manager_window(state)
                .and_then(|id| state.file_manager(id))
        });

    let dialog = manager
        .map(|manager| &manager.dialog)
        .unwrap_or(&FileManagerDialog::None);

    let action = match dialog {
        FileManagerDialog::DeleteConfirm { .. } => delete_confirm_key(key),
        FileManagerDialog::Rename { .. }
        | FileManagerDialog::Create { .. }
        | FileManagerDialog::GoToPath { .. } => dialog_input_key(key),
        FileManagerDialog::None => main_key(key),
    };

    match action {
        Some(action) => AppKeyResult::Action(action),
        None => AppKeyResult::Consumed,
    }
}

fn delete_confirm_key(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Esc => Some(Action::FileManager(
            FileManagerAction::FileManagerCancelDialog,
        )),
        KeyCode::Char('n' | 'N') => Some(Action::FileManager(
            FileManagerAction::FileManagerCancelDialog,
        )),
        KeyCode::Char('y' | 'Y') | KeyCode::Enter => Some(Action::FileManager(
            FileManagerAction::FileManagerConfirmDelete,
        )),
        _ => None,
    }
}

fn dialog_input_key(key: KeyEvent) -> Option<Action> {
    match key {
        KeyEvent {
            code: KeyCode::Esc, ..
        } => Some(Action::FileManager(
            FileManagerAction::FileManagerCancelDialog,
        )),
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => Some(Action::FileManager(
            FileManagerAction::FileManagerDialogCommit,
        )),
        KeyEvent {
            code: KeyCode::Backspace,
            ..
        } => Some(Action::FileManager(
            FileManagerAction::FileManagerDialogBackspace,
        )),
        KeyEvent {
            code: KeyCode::Char(character),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::FileManager(
            FileManagerAction::FileManagerDialogPush(character),
        )),
        _ => None,
    }
}

fn main_key(key: KeyEvent) -> Option<Action> {
    if file_manager_rename_shortcut(key) {
        return Some(Action::FileManager(
            FileManagerAction::FileManagerBeginRename,
        ));
    }
    match key {
        KeyEvent {
            code: KeyCode::Up, ..
        } => Some(Action::FileManager(FileManagerAction::MoveFileSelection(
            -1,
        ))),
        KeyEvent {
            code: KeyCode::Down,
            ..
        } => Some(Action::FileManager(FileManagerAction::MoveFileSelection(1))),
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => Some(Action::FileManager(FileManagerAction::OpenSelectedEntry)),
        KeyEvent {
            code: KeyCode::Backspace,
            ..
        }
        | KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::ALT,
            ..
        } => Some(Action::FileManager(FileManagerAction::FileManagerGoBack)),
        KeyEvent {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::FileManager(FileManagerAction::FileManagerGoUp)),
        KeyEvent {
            code: KeyCode::Char('g'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Char('~'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::FileManager(FileManagerAction::FileManagerGoHome)),
        KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::FileManager(
            FileManagerAction::FileManagerTogglePane,
        )),
        KeyEvent {
            code: KeyCode::PageUp,
            ..
        } => Some(Action::FileManager(
            FileManagerAction::FileManagerPageScroll(-1),
        )),
        KeyEvent {
            code: KeyCode::PageDown,
            ..
        } => Some(Action::FileManager(
            FileManagerAction::FileManagerPageScroll(1),
        )),
        KeyEvent {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::FileManager(
            FileManagerAction::ToggleFileManagerHidden,
        )),
        KeyEvent {
            code: KeyCode::Char('r'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::FileManager(FileManagerAction::ReloadFileManager)),
        KeyEvent {
            code: KeyCode::Char('1'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::FileManager(FileManagerAction::FileManagerSetSort(
            SortColumn::Name,
        ))),
        KeyEvent {
            code: KeyCode::Char('2'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::FileManager(FileManagerAction::FileManagerSetSort(
            SortColumn::Size,
        ))),
        KeyEvent {
            code: KeyCode::Char('3'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::FileManager(FileManagerAction::FileManagerSetSort(
            SortColumn::Modified,
        ))),
        KeyEvent {
            code: KeyCode::Char(':'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::FileManager(
            FileManagerAction::FileManagerBeginGoToPath,
        )),
        KeyEvent {
            code: KeyCode::Char('o'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::FileManager(
            FileManagerAction::FileManagerOpenInTerminal,
        )),
        KeyEvent {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::FileManager(
            FileManagerAction::FileManagerOpenInViewer,
        )),
        KeyEvent {
            code: KeyCode::Char('O'),
            modifiers: KeyModifiers::SHIFT,
            ..
        } => Some(Action::FileManager(
            FileManagerAction::FileManagerOpenWithSystem,
        )),
        KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Delete,
            ..
        } => Some(Action::FileManager(
            FileManagerAction::FileManagerRequestDelete,
        )),
        KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::FileManager(
            FileManagerAction::FileManagerBeginCreate(CreateKind::File),
        )),
        KeyEvent {
            code: KeyCode::Char('A'),
            modifiers: KeyModifiers::SHIFT,
            ..
        } => Some(Action::FileManager(
            FileManagerAction::FileManagerBeginCreate(CreateKind::Directory),
        )),
        other => quick_launch_action(other),
    }
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    window: &Window,
    interactions: &mut InteractionMap,
) {
    crate::ui::file_manager::render(frame, area, state, window.id, interactions);
}

pub fn palette_entries(state: &AppState) -> Vec<PaletteEntry> {
    if crate::app::file_manager::target_file_manager_window(state).is_none() {
        return Vec::new();
    }
    vec![
        PaletteEntry {
            title: "Toggle hidden files".to_owned(),
            detail: "File manager · show or hide dotfiles".to_owned(),
            action: Action::FileManager(FileManagerAction::ToggleFileManagerHidden),
        },
        PaletteEntry {
            title: "Reload directory".to_owned(),
            detail: "File manager · refresh listing".to_owned(),
            action: Action::FileManager(FileManagerAction::ReloadFileManager),
        },
        PaletteEntry {
            title: "Go to path".to_owned(),
            detail: "File manager · : then type path".to_owned(),
            action: Action::FileManager(FileManagerAction::FileManagerBeginGoToPath),
        },
        PaletteEntry {
            title: "Open in text viewer".to_owned(),
            detail: "File manager · e on selected file".to_owned(),
            action: Action::FileManager(FileManagerAction::FileManagerOpenInViewer),
        },
        PaletteEntry {
            title: "Open with system".to_owned(),
            detail: "File manager · Shift+O xdg-open / open".to_owned(),
            action: Action::FileManager(FileManagerAction::FileManagerOpenWithSystem),
        },
        PaletteEntry {
            title: "Open folder in terminal".to_owned(),
            detail: "File manager · o opens a new terminal in the selected folder".to_owned(),
            action: Action::FileManager(FileManagerAction::FileManagerOpenInTerminal),
        },
        PaletteEntry {
            title: "Trash or delete".to_owned(),
            detail: "File manager · remove selection (confirm)".to_owned(),
            action: Action::FileManager(FileManagerAction::FileManagerRequestDelete),
        },
        PaletteEntry {
            title: "Rename".to_owned(),
            detail: "File manager · Shift+R rename selection".to_owned(),
            action: Action::FileManager(FileManagerAction::FileManagerBeginRename),
        },
        PaletteEntry {
            title: "New file".to_owned(),
            detail: "File manager · create in current folder".to_owned(),
            action: Action::FileManager(FileManagerAction::FileManagerBeginCreate(
                CreateKind::File,
            )),
        },
        PaletteEntry {
            title: "New folder".to_owned(),
            detail: "File manager · create in current folder".to_owned(),
            action: Action::FileManager(FileManagerAction::FileManagerBeginCreate(
                CreateKind::Directory,
            )),
        },
    ]
}

fn open_path_with_system(path: &std::path::Path) -> Result<(), String> {
    use std::process::Command;
    #[cfg(target_os = "macos")]
    let status = Command::new("open").arg(path).status();
    #[cfg(target_os = "windows")]
    let status = Command::new("cmd")
        .args(["/C", "start", "", path.to_string_lossy().as_ref()])
        .status();
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let status = Command::new("xdg-open").arg(path).status();
    match status {
        Ok(exit) if exit.success() => Ok(()),
        Ok(exit) => Err(format!("exit code {}", exit)),
        Err(error) => Err(error.to_string()),
    }
}

pub async fn run_effect(
    executor: &mut crate::app::runtime::EffectExecutor,
    state: &mut AppState,
    effect: FileManagerEffect,
) {
    use crate::app::effects::FileManagerEffect;
    match effect {
        FileManagerEffect::ReadDirectory(window_id, path) => {
            executor.read_directory(state, window_id, path).await;
        }
        FileManagerEffect::DeletePath(window_id, path) => {
            let directory = state
                .file_manager(window_id)
                .map(|manager| manager.current_path.clone());
            let result = match executor.machine_for_window(state, window_id) {
                Some(machine) => machine.filesystem.remove_path(&path).await,
                None => Err(crate::machine::CapabilityError::Failed(
                    machine_unavailable_message(state, window_id),
                )),
            };
            if let Some(manager) = state.file_manager_mut(window_id) {
                manager.dialog = FileManagerDialog::None;
            }
            match result {
                Ok(crate::machine::RemoveOutcome::MovedToTrash) => {
                    state.status = format!("Moved to trash: {}", path.display());
                    if let Some(directory) = directory {
                        executor.read_directory(state, window_id, directory).await;
                    }
                }
                Ok(crate::machine::RemoveOutcome::DeletedPermanently) => {
                    state.status = format!("Deleted permanently: {}", path.display());
                    if let Some(directory) = directory {
                        executor.read_directory(state, window_id, directory).await;
                    }
                }
                Err(error) => {
                    state.status = format!("Remove failed: {error}");
                }
            }
        }
        FileManagerEffect::CreateEntry(window_id, path, kind) => {
            let directory = state
                .file_manager(window_id)
                .map(|manager| manager.current_path.clone());
            let result = match executor.machine_for_window(state, window_id) {
                Some(machine) => match kind {
                    CreateKind::File => machine.filesystem.create_file(&path).await,
                    CreateKind::Directory => machine.filesystem.create_directory(&path).await,
                },
                None => Err(crate::machine::CapabilityError::Failed(
                    machine_unavailable_message(state, window_id),
                )),
            };
            if let Some(manager) = state.file_manager_mut(window_id) {
                manager.dialog = FileManagerDialog::None;
            }
            match result {
                Ok(()) => {
                    state.status = format!("Created {}", path.display());
                    if let Some(directory) = directory {
                        executor.read_directory(state, window_id, directory).await;
                    }
                }
                Err(error) => state.status = format!("Create failed: {error}"),
            }
        }
        FileManagerEffect::OpenWithSystem(path) => {
            let display = path.display().to_string();
            let result = tokio::task::spawn_blocking(move || open_path_with_system(&path)).await;
            match result {
                Ok(Ok(())) => state.status = format!("Opened {display}"),
                Ok(Err(error)) => state.status = format!("Open failed: {error}"),
                Err(error) => state.status = format!("Open failed: {error}"),
            }
        }
        FileManagerEffect::RenamePath(window_id, from, to) => {
            let directory = state
                .file_manager(window_id)
                .map(|manager| manager.current_path.clone());
            let result = match executor.machine_for_window(state, window_id) {
                Some(machine) => machine.filesystem.rename_path(&from, &to).await,
                None => Err(crate::machine::CapabilityError::Failed(
                    machine_unavailable_message(state, window_id),
                )),
            };
            if let Some(manager) = state.file_manager_mut(window_id) {
                manager.dialog = FileManagerDialog::None;
            }
            match result {
                Ok(()) => {
                    state.status = format!(
                        "Renamed {} → {}",
                        from.file_name()
                            .map(|name| name.to_string_lossy())
                            .unwrap_or_default(),
                        to.file_name()
                            .map(|name| name.to_string_lossy())
                            .unwrap_or_default()
                    );
                    if let Some(directory) = directory {
                        executor.read_directory(state, window_id, directory).await;
                    }
                }
                Err(error) => {
                    state.status = format!("Rename failed: {error}");
                }
            }
        }
    }
}

fn machine_unavailable_message(state: &AppState, window_id: WindowId) -> String {
    let machine_id = state
        .window_machine_id(window_id)
        .unwrap_or_else(|| state.active_machine_id.clone());
    match machine_id {
        crate::machine::MachineId::Local => "local machine unavailable".to_owned(),
        crate::machine::MachineId::Named(name) => format!("not connected to {name}"),
    }
}
