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
    title: "Text Viewer",
    short_title: "View",
    description: "Read text files",
    quick_launch_key: Some('v'),
    on_open,
    on_close,
    on_open_window: None,
    render,
    dispatch_key,
    desktop_scroll: Some(|delta| Action::TextViewer(TextViewerAction::PageScroll(delta))),
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
    let dialog_active = state
        .focused_window()
        .filter(|window| window.application == ApplicationKind::TextViewer)
        .and_then(|window| state.text_viewer(window.id))
        .is_some_and(|view| matches!(view.dialog, TextViewerDialog::OpenPath { .. }));

    let action = if dialog_active {
        dialog_key(key)
    } else {
        main_key(key)
    };
    match action {
        Some(action) => AppKeyResult::Action(action),
        None => AppKeyResult::Consumed,
    }
}

fn main_key(key: KeyEvent) -> Option<Action> {
    match key {
        KeyEvent {
            code: KeyCode::Up, ..
        } => Some(Action::TextViewer(TextViewerAction::Scroll(-1))),
        KeyEvent {
            code: KeyCode::Down,
            ..
        } => Some(Action::TextViewer(TextViewerAction::Scroll(1))),
        KeyEvent {
            code: KeyCode::PageUp,
            ..
        } => Some(Action::TextViewer(TextViewerAction::PageScroll(-1))),
        KeyEvent {
            code: KeyCode::PageDown,
            ..
        } => Some(Action::TextViewer(TextViewerAction::PageScroll(1))),
        KeyEvent {
            code: KeyCode::Char(':'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::TextViewer(TextViewerAction::BeginOpenPath)),
        _ => None,
    }
}

fn dialog_key(key: KeyEvent) -> Option<Action> {
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

pub fn palette_extras(_state: &AppState) -> Vec<PaletteEntry> {
    vec![PaletteEntry {
        title: "Open file path".to_owned(),
        detail: "Text viewer · : then type path".to_owned(),
        action: Action::TextViewer(TextViewerAction::BeginOpenPath),
    }]
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
    }
}
