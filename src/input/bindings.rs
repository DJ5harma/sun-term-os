//! Single source of truth for keyboard bindings by scope.
//!
//! **Nested terminals** (GNOME VTE, VS Code, Windows Terminal, etc.) routinely steal
//! `Ctrl+digit`, `Alt+digit`, `Ctrl+Tab`, and many `F` keys before the application sees them.
//! Window focus therefore uses a **prefix chord** (`Ctrl+G` then `1–9`), the same pattern as
//! tmux/screen, plus mouse clicks on the bottom bar (always reliable).

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::actions::{Action, PaletteAction, ShellAction};

/// Shown in the shell chrome; keep in sync with [match_global].
pub const LAUNCHER_SHORTCUT_HINT: &str = "Alt+P command palette";

pub const WINDOW_FOCUS_HINT: &str = "Ctrl+G, then 1–9";

pub const WORKSPACE_HINT: &str = "F1–F9";

pub fn match_global(key: KeyEvent) -> Option<Action> {
    if !key.is_press() && !key.is_repeat() {
        return None;
    }
    if is_launcher(key) {
        return Some(Action::Palette(PaletteAction::ToggleLauncher));
    }
    if is_begin_window_pick(key) {
        return Some(Action::Shell(ShellAction::BeginWindowPick));
    }
    if let Some(action) = window_chrome(key) {
        return Some(action);
    }
    if is_input_debug_toggle(key) {
        return Some(Action::Shell(ShellAction::ToggleInputDebug));
    }
    workspace_from_function_key(key.code)
        .map(|index| Action::Shell(ShellAction::SwitchWorkspace(index)))
}

/// Second step of [WINDOW_FOCUS_HINT] (only while `window_pick_mode` is active).
pub fn match_window_pick(key: KeyEvent) -> Option<Action> {
    if !key.is_press() && !key.is_repeat() {
        return None;
    }
    if matches!(key.code, KeyCode::Esc) {
        return Some(Action::Shell(ShellAction::CancelWindowPick));
    }
    if is_begin_window_pick(key) {
        return Some(Action::Shell(ShellAction::CancelWindowPick));
    }
    let KeyCode::Char(character) = key.code else {
        return None;
    };
    if key.modifiers != KeyModifiers::NONE {
        return None;
    }
    if !character.is_ascii_digit() {
        return None;
    }
    let slot = character.to_digit(10)? as u8;
    (1..=9)
        .contains(&slot)
        .then_some(Action::Shell(ShellAction::FocusWindowSlot(slot)))
}

fn is_input_debug_toggle(key: KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL)
        && key.modifiers.contains(KeyModifiers::ALT)
        && matches!(key.code, KeyCode::Char('d' | 'D'))
}

pub fn global_consumes(key: KeyEvent) -> bool {
    match_global(key).is_some()
}

pub fn match_modal(key: KeyEvent) -> Option<Action> {
    if !key.is_press() && !key.is_repeat() {
        return None;
    }
    match key.code {
        KeyCode::Esc => Some(Action::Palette(PaletteAction::CloseLauncher)),
        KeyCode::Up => Some(Action::Palette(PaletteAction::MoveLauncherUp)),
        KeyCode::Down => Some(Action::Palette(PaletteAction::MoveLauncherDown)),
        KeyCode::Enter => Some(Action::Palette(PaletteAction::ExecuteLauncherSelection)),
        KeyCode::Backspace => Some(Action::Palette(PaletteAction::PaletteQueryBackspace)),
        KeyCode::Char(character) if key.modifiers == KeyModifiers::NONE => {
            Some(Action::Palette(PaletteAction::PaletteQueryPush(character)))
        }
        _ => None,
    }
}

fn is_launcher(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('p' | 'P'))
        && key.modifiers.intersects(KeyModifiers::ALT)
        && !key.modifiers.intersects(KeyModifiers::CONTROL)
}

fn is_begin_window_pick(key: KeyEvent) -> bool {
    key.modifiers == KeyModifiers::CONTROL && matches!(key.code, KeyCode::Char('g' | 'G'))
}

fn window_chrome(key: KeyEvent) -> Option<Action> {
    if !key.modifiers.intersects(KeyModifiers::CONTROL)
        || key.modifiers.intersects(KeyModifiers::SHIFT)
    {
        return None;
    }
    match key.code {
        KeyCode::Char('w' | 'W') => Some(Action::Shell(ShellAction::CloseWindow)),
        KeyCode::Char('m' | 'M') => Some(Action::Shell(ShellAction::MinimizeWindow)),
        KeyCode::Char('f' | 'F') => Some(Action::Shell(ShellAction::ToggleMaximizeWindow)),
        _ => None,
    }
}

fn workspace_from_function_key(code: KeyCode) -> Option<usize> {
    match code {
        KeyCode::F(index) if (1..=9).contains(&index) => Some(index as usize - 1),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEvent;

    #[test]
    fn ctrl_g_begins_window_pick() {
        assert_eq!(
            match_global(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL)),
            Some(Action::Shell(ShellAction::BeginWindowPick))
        );
    }

    #[test]
    fn pick_mode_digit_focuses_slot() {
        assert_eq!(
            match_window_pick(KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE)),
            Some(Action::Shell(ShellAction::FocusWindowSlot(3)))
        );
    }
}
