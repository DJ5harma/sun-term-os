use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Frame, layout::Rect};

use crate::{
    actions::{Action, AsyncAction, TextViewerAction},
    app::{
        effects::{Effect, TextViewerEffect},
        palette::PaletteEntry,
        state::AppState,
        text_viewer::TextViewerDialog,
    },
    domain::{ApplicationKind, Window, WindowId},
    ui::interaction::InteractionMap,
};

use super::{AppKeyResult, BuiltInApp};

pub const APP: BuiltInApp = BuiltInApp {
    kind: ApplicationKind::TextViewer,
    title: "Notepad",
    short_title: "Note",
    description: "Edit text files",
    quick_launch_key: Some('v'),
    on_open,
    on_close,
    on_open_window: None,
    render,
    dispatch_key,
    desktop_scroll: Some(|delta| Action::TextViewer(TextViewerAction::ScrollView(delta))),
    palette_extras: Some(palette_extras),
};

pub fn on_open(state: &mut AppState, window_id: WindowId) -> Vec<Effect> {
    state.init_text_viewer_empty(window_id);
    Vec::new()
}

pub fn on_close(state: &mut AppState, window_id: WindowId) -> Vec<Effect> {
    state.remove_text_viewer(window_id);
    Vec::new()
}

pub fn dispatch_key(key: KeyEvent, state: &AppState) -> AppKeyResult {
    let dialog = state
        .focused_window()
        .filter(|window| window.application == ApplicationKind::TextViewer)
        .and_then(|window| state.text_viewer(window.id))
        .map(|view| view.dialog.clone());

    let action = match dialog {
        Some(TextViewerDialog::OpenPath { .. }) => open_path_dialog_key(key),
        Some(TextViewerDialog::ConfirmDiscardClose) => discard_close_dialog_key(key),
        _ => main_key(key),
    };
    match action {
        Some(action) => AppKeyResult::Action(action),
        None => AppKeyResult::Consumed,
    }
}

fn main_key(key: KeyEvent) -> Option<Action> {
    if key.modifiers == KeyModifiers::CONTROL && matches!(key.code, KeyCode::Char('s' | 'S')) {
        return Some(Action::TextViewer(TextViewerAction::Save));
    }

    match key.code {
        KeyCode::Up => Some(Action::TextViewer(TextViewerAction::CursorUp)),
        KeyCode::Down => Some(Action::TextViewer(TextViewerAction::CursorDown)),
        KeyCode::Left => Some(Action::TextViewer(TextViewerAction::CursorLeft)),
        KeyCode::Right => Some(Action::TextViewer(TextViewerAction::CursorRight)),
        KeyCode::Home => Some(Action::TextViewer(TextViewerAction::CursorHome)),
        KeyCode::End => Some(Action::TextViewer(TextViewerAction::CursorEnd)),
        KeyCode::PageUp => Some(Action::TextViewer(TextViewerAction::PageScroll(-1))),
        KeyCode::PageDown => Some(Action::TextViewer(TextViewerAction::PageScroll(1))),
        KeyCode::Backspace => Some(Action::TextViewer(TextViewerAction::Backspace)),
        KeyCode::Delete => Some(Action::TextViewer(TextViewerAction::Delete)),
        KeyCode::Char(':') if key.modifiers == KeyModifiers::NONE => {
            Some(Action::TextViewer(TextViewerAction::BeginOpenPath))
        }
        KeyCode::Char(ch) if key.modifiers == KeyModifiers::NONE => {
            Some(Action::TextViewer(TextViewerAction::InsertChar(ch)))
        }
        KeyCode::Char(ch) if key.modifiers == KeyModifiers::SHIFT => {
            Some(Action::TextViewer(TextViewerAction::InsertChar(ch)))
        }
        KeyCode::Tab if key.modifiers == KeyModifiers::NONE => {
            Some(Action::TextViewer(TextViewerAction::InsertTab))
        }
        _ => None,
    }
}

fn open_path_dialog_key(key: KeyEvent) -> Option<Action> {
    match key {
        KeyEvent {
            code: KeyCode::Esc, ..
        } => Some(Action::TextViewer(TextViewerAction::DialogCancel)),
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => Some(Action::TextViewer(TextViewerAction::DialogCommit)),
        KeyEvent {
            code: KeyCode::Backspace,
            ..
        } => Some(Action::TextViewer(TextViewerAction::DialogBackspace)),
        KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::TextViewer(TextViewerAction::DialogPush(ch))),
        _ => None,
    }
}

fn discard_close_dialog_key(key: KeyEvent) -> Option<Action> {
    if key.modifiers == KeyModifiers::CONTROL && matches!(key.code, KeyCode::Char('s' | 'S')) {
        return Some(Action::TextViewer(TextViewerAction::Save));
    }
    match key.code {
        KeyCode::Esc => Some(Action::TextViewer(TextViewerAction::DialogCancel)),
        KeyCode::Char('d' | 'D') if key.modifiers == KeyModifiers::NONE => {
            Some(Action::TextViewer(TextViewerAction::DiscardAndClose))
        }
        _ => None,
    }
}

pub fn palette_extras(_state: &AppState) -> Vec<PaletteEntry> {
    vec![
        PaletteEntry {
            title: "Save file".to_owned(),
            detail: "Notepad · Ctrl+S".to_owned(),
            action: Action::TextViewer(TextViewerAction::Save),
        },
        PaletteEntry {
            title: "Open file path".to_owned(),
            detail: "Notepad · : then type path".to_owned(),
            action: Action::TextViewer(TextViewerAction::BeginOpenPath),
        },
    ]
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    window: &Window,
    _interactions: &mut InteractionMap,
) {
    if let Some(view) = state.text_viewer(window.id) {
        crate::ui::text_viewer::render(frame, area, state, window.id, view);
    }
}

pub async fn run_effect(
    executor: &mut crate::app::runtime::EffectExecutor,
    state: &mut AppState,
    effect: TextViewerEffect,
) {
    match effect {
        TextViewerEffect::Read(window_id, path) => {
            const MAX: u64 = 4 * 1024 * 1024;
            let result = match executor.machine_for_window(state, window_id) {
                Some(machine) => machine
                    .filesystem
                    .read_text_file(&path, MAX)
                    .await
                    .map_err(|error| error.to_string()),
                None => {
                    let machine_id = state
                        .window_machine_id(window_id)
                        .unwrap_or_else(|| state.active_machine_id.clone());
                    Err(match machine_id {
                        crate::machine::MachineId::Local => "local machine unavailable".to_owned(),
                        crate::machine::MachineId::Named(name) => {
                            format!("not connected to {name}")
                        }
                    })
                }
            };
            let result = result
                .map_err(|error| crate::app::offline::enrich_load_error(state, window_id, &error));
            crate::app::reducer::reduce(
                state,
                Action::Async(AsyncAction::TextFileReady(window_id, result)),
            );
        }
        TextViewerEffect::Write(window_id, path, contents) => {
            let result = match executor.machine_for_window(state, window_id) {
                Some(machine) => machine
                    .filesystem
                    .write_text_file(&path, &contents)
                    .await
                    .map_err(|error| error.to_string()),
                None => {
                    let machine_id = state
                        .window_machine_id(window_id)
                        .unwrap_or_else(|| state.active_machine_id.clone());
                    Err(match machine_id {
                        crate::machine::MachineId::Local => "local machine unavailable".to_owned(),
                        crate::machine::MachineId::Named(name) => {
                            format!("not connected to {name}")
                        }
                    })
                }
            };
            let result = result
                .map_err(|error| crate::app::offline::enrich_load_error(state, window_id, &error));
            crate::app::reducer::reduce(
                state,
                Action::Async(AsyncAction::TextFileSaveReady(window_id, result)),
            );
        }
    }
}
