use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Frame, layout::Rect};

use crate::{
    actions::{Action, TerminalAction},
    app::{
        AppState, TerminalStatus,
        effects::{Effect, TerminalEffect},
    },
    domain::{ApplicationKind, Window, WindowId},
    input::terminal_encode::encode_key,
    ui::interaction::InteractionMap,
};

use super::{AppKeyResult, BuiltInApp};

pub const APP: BuiltInApp = BuiltInApp {
    kind: ApplicationKind::Terminal,
    title: "Terminal",
    short_title: "Term",
    description: "Run commands and manage shells",
    quick_launch_key: Some('t'),
    on_open,
    on_close,
    on_open_window: Some(on_open_window),
    render,
    dispatch_key,
    desktop_scroll: None,
    palette_extras: None,
};

pub fn on_open(_state: &mut AppState, window_id: WindowId) -> Vec<Effect> {
    vec![Effect::Terminal(TerminalEffect::Start(window_id))]
}

pub fn on_close(_state: &mut AppState, window_id: WindowId) -> Vec<Effect> {
    vec![Effect::Terminal(TerminalEffect::Stop(window_id))]
}

pub fn on_open_window(state: &mut AppState, window_id: WindowId) {
    state.set_terminal_status(window_id, TerminalStatus::Starting);
}

pub fn dispatch_key(key: KeyEvent, _state: &AppState) -> AppKeyResult {
    if key.modifiers.contains(KeyModifiers::ALT) {
        match key.code {
            KeyCode::PageUp => {
                return AppKeyResult::Action(Action::Terminal(TerminalAction::ScrollOutput(-3)));
            }
            KeyCode::PageDown => {
                return AppKeyResult::Action(Action::Terminal(TerminalAction::ScrollOutput(3)));
            }
            KeyCode::End => {
                return AppKeyResult::Action(Action::Terminal(TerminalAction::ScrollToEnd));
            }
            _ => {}
        }
    }
    if let Some(bytes) = encode_key(key) {
        AppKeyResult::Terminal(bytes)
    } else {
        AppKeyResult::Consumed
    }
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    window: &Window,
    _interactions: &mut InteractionMap,
) {
    crate::ui::terminal::render(frame, area, state, window.id);
}

pub async fn run_effect(
    executor: &mut crate::app::runtime::EffectExecutor,
    state: &mut AppState,
    effect: TerminalEffect,
) {
    use crate::app::effects::TerminalEffect;
    match effect {
        TerminalEffect::Start(window_id) => {
            if let Err(error) = executor.terminal_manager.open(window_id, 80, 24) {
                state.set_terminal_status(window_id, TerminalStatus::Failed(error.to_string()));
                state.status = format!("Terminal failed to start: {error}");
            } else {
                state.set_terminal_status(window_id, TerminalStatus::Running);
            }
        }
        TerminalEffect::Stop(window_id) => {
            executor.terminal_manager.close(window_id);
            state.remove_terminal_content(window_id);
            state.remove_terminal_status(window_id);
        }
        TerminalEffect::Write(window_id, bytes) => {
            let _ = executor.terminal_manager.write_input(window_id, &bytes);
        }
    }
}
