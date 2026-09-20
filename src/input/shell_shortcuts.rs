//! Desktop shell shortcuts that must win over application input.

use crossterm::event::{KeyCode, KeyEvent};

use crate::actions::Action;

/// Returns a shell action if this key event should be handled before application input.
pub fn resolve(key: KeyEvent) -> Option<Action> {
    if !key.is_press() && !key.is_repeat() {
        return None;
    }
    workspace_from_function_key(key.code).map(Action::SwitchWorkspace)
}

pub fn consumes_for_shell(key: KeyEvent) -> bool {
    resolve(key).is_some()
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
    fn only_f_keys_switch_workspaces() {
        assert_eq!(
            resolve(KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE)),
            Some(Action::SwitchWorkspace(1))
        );
        assert_eq!(
            resolve(KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE)),
            None
        );
        assert_eq!(
            resolve(KeyEvent::new(KeyCode::PageDown, KeyModifiers::CONTROL)),
            None
        );
    }

    #[test]
    fn key_release_is_ignored() {
        let key = KeyEvent::new_with_kind(KeyCode::F(1), KeyModifiers::NONE, KeyEventKind::Release);
        assert_eq!(resolve(key), None);
    }
}
