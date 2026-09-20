use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;

use crate::{
    actions::Action,
    app::{AppState, ApplicationKind},
    ui::geometry::UiGeometry,
};

pub fn action_for_key(
    key: KeyEvent,
    launcher_open: bool,
    terminal_focused: bool,
) -> Option<Action> {
    if launcher_open {
        return match key.code {
            KeyCode::Esc => Some(Action::CloseLauncher),
            KeyCode::Up => Some(Action::MoveLauncherUp),
            KeyCode::Down => Some(Action::MoveLauncherDown),
            KeyCode::Enter => Some(Action::ExecuteLauncherSelection),
            _ => None,
        };
    }

    if terminal_focused {
        return match key {
            KeyEvent {
                code: KeyCode::Char('w'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => Some(Action::CloseWindow),
            KeyEvent {
                code: KeyCode::Char('m'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => Some(Action::MinimizeWindow),
            KeyEvent {
                code: KeyCode::Char('f'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => Some(Action::ToggleMaximizeWindow),
            KeyEvent {
                code: KeyCode::Char(number @ '1'..='3'),
                modifiers,
                ..
            } if modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::SwitchWorkspace(number as usize - '1' as usize))
            }
            _ => None,
        };
    }

    match key {
        KeyEvent {
            code: KeyCode::Char('q'),
            ..
        }
        | KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => Some(Action::Quit),
        KeyEvent {
            code: KeyCode::Char('p'),
            ..
        }
        | KeyEvent {
            code: KeyCode::Char(':'),
            ..
        } => Some(Action::ToggleLauncher),
        KeyEvent {
            code: KeyCode::Tab, ..
        } => Some(Action::FocusNextWindow),
        KeyEvent {
            code: KeyCode::BackTab,
            ..
        } => Some(Action::FocusPreviousWindow),
        KeyEvent {
            code: KeyCode::Char('r'),
            ..
        } => Some(Action::Refresh),
        KeyEvent {
            code: KeyCode::Char('t'),
            ..
        } => Some(Action::OpenApplication(ApplicationKind::Terminal)),
        KeyEvent {
            code: KeyCode::Char('w'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => Some(Action::CloseWindow),
        KeyEvent {
            code: KeyCode::Char('m'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => Some(Action::MinimizeWindow),
        KeyEvent {
            code: KeyCode::Char('f'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => Some(Action::ToggleMaximizeWindow),
        KeyEvent {
            code: KeyCode::Char(number @ '1'..='3'),
            ..
        } => Some(Action::SwitchWorkspace(number as usize - '1' as usize)),
        _ => None,
    }
}

pub fn terminal_input(key: KeyEvent) -> Option<Vec<u8>> {
    if key.modifiers.contains(KeyModifiers::CONTROL)
        && let KeyCode::Char(character) = key.code
    {
        let character = character.to_ascii_lowercase();
        if character.is_ascii_lowercase() {
            return Some(vec![character as u8 - b'a' + 1]);
        }
    }
    let bytes = match key.code {
        KeyCode::Char(character) => character.to_string().into_bytes(),
        KeyCode::Enter => b"\r".to_vec(),
        KeyCode::Backspace => vec![0x7f],
        KeyCode::Tab => b"\t".to_vec(),
        KeyCode::Esc => vec![0x1b],
        KeyCode::Up => b"\x1b[A".to_vec(),
        KeyCode::Down => b"\x1b[B".to_vec(),
        KeyCode::Right => b"\x1b[C".to_vec(),
        KeyCode::Left => b"\x1b[D".to_vec(),
        KeyCode::Home => b"\x1b[H".to_vec(),
        KeyCode::End => b"\x1b[F".to_vec(),
        KeyCode::Delete => b"\x1b[3~".to_vec(),
        _ => return None,
    };
    Some(bytes)
}

pub fn action_for_mouse(
    mouse: MouseEvent,
    state: &AppState,
    geometry: &UiGeometry,
) -> Option<Action> {
    let position = Position::new(mouse.column, mouse.row);
    if state.launcher_open {
        if mouse.kind == MouseEventKind::ScrollUp {
            return Some(Action::MoveLauncherUp);
        }
        if mouse.kind == MouseEventKind::ScrollDown {
            return Some(Action::MoveLauncherDown);
        }
        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return None;
        }
        if !geometry.launcher.contains(position) {
            return Some(Action::CloseLauncher);
        }
        for (application, area) in &geometry.launcher_targets {
            if area.contains(position) {
                return Some(Action::OpenApplication(*application));
            }
        }
        return None;
    }
    if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
        return None;
    }
    if geometry.launcher_button.contains(position) {
        return Some(Action::ToggleLauncher);
    }
    for (index, area) in &geometry.workspace_targets {
        if area.contains(position) {
            return Some(Action::SwitchWorkspace(*index));
        }
    }
    for (id, area) in &geometry.window_targets {
        if area.contains(position)
            && state
                .current_workspace()
                .windows
                .iter()
                .any(|window| window.id == *id)
        {
            return Some(Action::FocusWindow(*id));
        }
    }
    None
}

pub fn terminal_mouse(mouse: MouseEvent, geometry: &UiGeometry) -> Option<Vec<u8>> {
    let position = Position::new(mouse.column, mouse.row);
    if !geometry.desktop.contains(position) {
        return None;
    }
    let column = mouse.column.saturating_sub(geometry.desktop.x).max(1);
    let row = mouse.row.saturating_sub(geometry.desktop.y).max(1);
    let (button, suffix) = match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => (0, 'M'),
        MouseEventKind::Down(MouseButton::Middle) => (1, 'M'),
        MouseEventKind::Down(MouseButton::Right) => (2, 'M'),
        MouseEventKind::Up(MouseButton::Left) => (0, 'm'),
        MouseEventKind::Up(MouseButton::Middle) => (1, 'm'),
        MouseEventKind::Up(MouseButton::Right) => (2, 'm'),
        MouseEventKind::Drag(MouseButton::Left) => (32, 'M'),
        MouseEventKind::Drag(MouseButton::Middle) => (33, 'M'),
        MouseEventKind::Drag(MouseButton::Right) => (34, 'M'),
        MouseEventKind::ScrollUp => (64, 'M'),
        MouseEventKind::ScrollDown => (65, 'M'),
        MouseEventKind::ScrollLeft => (66, 'M'),
        MouseEventKind::ScrollRight => (67, 'M'),
        _ => return None,
    };
    Some(format!("\x1b[<{};{};{}{}", button, column, row, suffix).into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn ctrl_c_is_forwarded_to_a_focused_terminal() {
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);

        assert_eq!(action_for_key(key, false, true), None);
        assert_eq!(terminal_input(key), Some(vec![3]));
    }

    #[test]
    fn launcher_and_terminal_shortcuts_are_translated() {
        assert_eq!(
            action_for_key(
                KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE),
                false,
                false
            ),
            Some(Action::ToggleLauncher)
        );
        assert_eq!(
            action_for_key(
                KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE),
                false,
                false
            ),
            Some(Action::OpenApplication(ApplicationKind::Terminal))
        );
    }

    #[test]
    fn launcher_navigation_does_not_leak_to_desktop_actions() {
        assert_eq!(
            action_for_key(
                KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
                true,
                false
            ),
            Some(Action::MoveLauncherDown)
        );
        assert_eq!(
            action_for_key(
                KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
                true,
                false
            ),
            None
        );
    }

    #[test]
    fn mouse_workspace_hit_testing_uses_rendered_geometry() {
        let state = AppState::default();
        let geometry = crate::ui::geometry::calculate(Rect::new(0, 0, 120, 40), &state);
        let target = geometry.workspace_targets[1].1;
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: target.x + 1,
            row: target.y + 1,
            modifiers: KeyModifiers::NONE,
        };

        assert_eq!(
            action_for_mouse(mouse, &state, &geometry),
            Some(Action::SwitchWorkspace(1))
        );
    }

    #[test]
    fn terminal_input_translates_control_and_navigation_keys() {
        assert_eq!(
            terminal_input(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Some(vec![3])
        );
        assert_eq!(
            terminal_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            Some(b"\r".to_vec())
        );
        assert_eq!(
            terminal_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
            Some(b"\x1b[A".to_vec())
        );
    }

    #[test]
    fn launcher_clicks_and_terminal_mouse_events_use_shared_geometry() {
        let state = AppState::default();
        let geometry = crate::ui::geometry::calculate(Rect::new(0, 0, 120, 40), &state);
        let workspace = geometry.workspace_targets[1].1;
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: workspace.x + 1,
            row: workspace.y + 1,
            modifiers: KeyModifiers::NONE,
        };
        assert_eq!(
            action_for_mouse(mouse, &state, &geometry),
            Some(Action::SwitchWorkspace(1))
        );

        let terminal_mouse_event = MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: geometry.desktop.x + 4,
            row: geometry.desktop.y + 3,
            modifiers: KeyModifiers::NONE,
        };
        assert_eq!(
            terminal_mouse(terminal_mouse_event, &geometry),
            Some(b"\x1b[<64;4;3M".to_vec())
        );
    }
}
