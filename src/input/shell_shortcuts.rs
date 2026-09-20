//! Desktop shell shortcuts that must win over application input.
//!
//! Launcher uses a **modifier chord** (`Ctrl+Shift+P`) so bare keys like `p`, `:`, `/`
//! stay available to terminals and TUIs. See also the always-visible **Apps** control in the top bar.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::actions::Action;

/// Shown in the shell chrome; keep in sync with [is_launcher_shortcut].
pub const LAUNCHER_SHORTCUT_HINT: &str = "Ctrl+Shift+P";

/// Returns a shell action if this key event should be handled before application input.
pub fn resolve(key: KeyEvent) -> Option<Action> {
    if !key.is_press() && !key.is_repeat() {
        return None;
    }
    if is_launcher_shortcut(key) {
        return Some(Action::ToggleLauncher);
    }
    if key.modifiers == KeyModifiers::CONTROL {
        return match key.code {
            KeyCode::Char('w') => Some(Action::CloseWindow),
            KeyCode::Char('m') => Some(Action::MinimizeWindow),
            KeyCode::Char('f') => Some(Action::ToggleMaximizeWindow),
            _ => workspace_from_function_key(key.code).map(Action::SwitchWorkspace),
        };
    }
    if key.modifiers == KeyModifiers::NONE {
        return workspace_from_function_key(key.code).map(Action::SwitchWorkspace);
    }
    workspace_from_function_key(key.code).map(Action::SwitchWorkspace)
}

pub fn consumes_for_shell(key: KeyEvent) -> bool {
    resolve(key).is_some()
}

pub fn is_launcher_shortcut(key: KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL)
        && key.modifiers.contains(KeyModifiers::SHIFT)
        && matches!(key.code, KeyCode::Char('p') | KeyCode::Char('P'))
}

fn workspace_from_function_key(code: KeyCode) -> Option<usize> {
    match code {
        KeyCode::F(1) => Some(0),
        KeyCode::F(2) => Some(1),
        KeyCode::F(3) => Some(2),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, KeyModifiers};

    #[test]
    fn f_keys_switch_workspaces() {
        assert_eq!(
            resolve(KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE)),
            Some(Action::SwitchWorkspace(1))
        );
        assert_eq!(
            resolve(KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE)),
            None
        );
    }

    #[test]
    fn bare_p_is_not_captured_by_the_shell() {
        assert_eq!(
            resolve(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE)),
            None
        );
        assert_eq!(
            resolve(KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE)),
            None
        );
    }

    #[test]
    fn launcher_uses_control_shift_p() {
        let chord = KeyEvent::new(
            KeyCode::Char('p'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );
        assert!(is_launcher_shortcut(chord));
        assert_eq!(resolve(chord), Some(Action::ToggleLauncher));
    }

    #[test]
    fn window_chrome_shortcuts_use_control_modifier() {
        assert_eq!(
            resolve(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL)),
            Some(Action::CloseWindow)
        );
    }

    #[test]
    fn key_release_is_ignored() {
        let key = KeyEvent::new_with_kind(KeyCode::F(1), KeyModifiers::NONE, KeyEventKind::Release);
        assert_eq!(resolve(key), None);
    }
}
