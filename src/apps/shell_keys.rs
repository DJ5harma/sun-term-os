use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    actions::{Action, ShellAction},
    app::AppState,
};

use super::{AppKeyResult, by_quick_launch_key, open_action};

pub fn dispatch_key(key: KeyEvent, _state: &AppState) -> AppKeyResult {
    match key {
        KeyEvent {
            code: KeyCode::Char('q'),
            ..
        }
        | KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => AppKeyResult::Action(Action::Shell(ShellAction::Quit)),
        KeyEvent {
            code: KeyCode::Char('r'),
            ..
        } => AppKeyResult::Action(Action::Shell(ShellAction::Refresh)),
        other => match quick_launch_action(other) {
            Some(action) => AppKeyResult::Action(action),
            None => AppKeyResult::Consumed,
        },
    }
}

/// Quick-open shortcuts when typing must not be stolen (non-terminal focus).
pub fn quick_launch_action(key: KeyEvent) -> Option<Action> {
    if key.modifiers != KeyModifiers::NONE {
        return None;
    }
    let KeyCode::Char(character) = key.code else {
        return None;
    };
    by_quick_launch_key(character).map(|app| open_action(app.kind))
}
