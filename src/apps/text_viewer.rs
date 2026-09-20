use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, layout::Rect};

use crate::{
    actions::{Action, AsyncAction, TextViewerAction},
    app::{
        effects::{Effect, TextViewerEffect},
        state::AppState,
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
    palette_extras: None,
};

pub fn on_open(_state: &mut AppState, _window_id: WindowId) -> Vec<Effect> {
    Vec::new()
}

pub fn on_close(state: &mut AppState, window_id: WindowId) -> Vec<Effect> {
    state.remove_text_viewer(window_id);
    Vec::new()
}

pub fn dispatch_key(key: KeyEvent, _state: &AppState) -> AppKeyResult {
    let action = match key {
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
        _ => None,
    };
    match action {
        Some(action) => AppKeyResult::Action(action),
        None => AppKeyResult::Consumed,
    }
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    window: &Window,
    _interactions: &mut InteractionMap,
) {
    if let Some(view) = state.text_viewer(window.id) {
        crate::ui::text_viewer::render(frame, area, view);
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
            crate::app::reducer::reduce(
                state,
                Action::Async(AsyncAction::TextFileReady(window_id, result)),
            );
        }
    }
}
