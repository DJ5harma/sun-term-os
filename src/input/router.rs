//! Layered keyboard dispatch: global bindings first, then modal / app layers.

use crossterm::event::KeyEvent;

use crate::{
    actions::Action,
    app::AppState,
    apps::{self, AppKeyResult, shell_keys},
};

use super::bindings::{match_global, match_modal, match_window_pick};
use super::normalize::normalize_key_event;

#[derive(Debug, Clone, PartialEq)]
pub enum KeyDispatch {
    Action(Action),
    Terminal(Vec<u8>),
    /// Key ignored without forwarding to a terminal.
    Consumed,
}

pub fn dispatch_key(key: KeyEvent, state: &AppState) -> KeyDispatch {
    if !key.is_press() && !key.is_repeat() {
        return KeyDispatch::Consumed;
    }

    let normalized = normalize_key_event(key);

    if state.window_pick_mode {
        return match match_window_pick(normalized) {
            Some(action) => KeyDispatch::Action(action),
            None => KeyDispatch::Consumed,
        };
    }

    if let Some(action) = match_global(normalized) {
        return KeyDispatch::Action(action);
    }

    if state.launcher_open {
        return match match_modal(normalized) {
            Some(action) => KeyDispatch::Action(action),
            None => KeyDispatch::Consumed,
        };
    }

    if state.shows_home_screen()
        && let Some(action) = apps::home_screen::dispatch_key(normalized, state)
    {
        return KeyDispatch::Action(action);
    }

    let dispatch = if let Some(window) = state.focused_window() {
        apps::dispatch_key(window.application, normalized, state)
    } else {
        shell_keys::dispatch_key(normalized, state)
    };

    match dispatch {
        AppKeyResult::Action(action) => KeyDispatch::Action(action),
        AppKeyResult::Terminal(bytes) => KeyDispatch::Terminal(bytes),
        AppKeyResult::Consumed => KeyDispatch::Consumed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::{FileManagerAction, ShellAction};
    use crate::app::reducer::reduce;
    use crate::app::{AppState, ApplicationKind};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn state_with_file_manager() -> AppState {
        let mut state = AppState::default();
        reduce(
            &mut state,
            Action::Shell(ShellAction::OpenApplication(ApplicationKind::FileManager)),
        );
        state
    }

    fn state_with_terminal() -> AppState {
        let mut state = AppState::default();
        reduce(
            &mut state,
            Action::Shell(ShellAction::OpenApplication(ApplicationKind::Terminal)),
        );
        state
    }

    #[test]
    fn ctrl_g_starts_pick_from_terminal() {
        let key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL);
        assert_eq!(
            dispatch_key(key, &state_with_terminal()),
            KeyDispatch::Action(Action::Shell(ShellAction::BeginWindowPick))
        );
    }

    #[test]
    fn digit_in_pick_mode_focuses_from_file_manager() {
        let mut state = state_with_file_manager();
        state.window_pick_mode = true;
        let key = KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE);
        assert_eq!(
            dispatch_key(key, &state),
            KeyDispatch::Action(Action::Shell(ShellAction::FocusWindowSlot(2)))
        );
    }

    #[test]
    fn bare_digit_does_not_focus_without_pick() {
        let key = KeyEvent::new(KeyCode::Char('8'), KeyModifiers::NONE);
        assert_eq!(
            dispatch_key(key, &state_with_file_manager()),
            KeyDispatch::Consumed
        );
    }

    #[test]
    fn file_manager_plain_tab_toggles_pane() {
        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert_eq!(
            dispatch_key(key, &state_with_file_manager()),
            KeyDispatch::Action(Action::FileManager(
                FileManagerAction::FileManagerTogglePane
            ))
        );
    }

    #[test]
    fn shift_r_begins_rename_in_file_manager() {
        let key = KeyEvent::new(KeyCode::Char('R'), KeyModifiers::SHIFT);
        assert_eq!(
            dispatch_key(key, &state_with_file_manager()),
            KeyDispatch::Action(Action::FileManager(
                FileManagerAction::FileManagerBeginRename
            ))
        );
    }

    #[test]
    fn enter_in_rename_dialog_commits() {
        let mut state = state_with_file_manager();
        let window_id = state.focused_window().unwrap().id;
        if let Some(manager) = state.file_manager_mut(window_id) {
            manager.dialog = crate::app::state::FileManagerDialog::Rename {
                path: std::path::PathBuf::from("/tmp/x"),
                input: "y".to_owned(),
            };
        }
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(
            dispatch_key(key, &state),
            KeyDispatch::Action(Action::FileManager(
                FileManagerAction::FileManagerDialogCommit
            ))
        );
    }
}
