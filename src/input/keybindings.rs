use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;

use crate::{
    actions::Action,
    app::{AppState, ApplicationKind},
    ui::geometry::UiGeometry,
};

pub fn action_for_key(key: KeyEvent, launcher_open: bool) -> Option<Action> {
    if launcher_open {
        return match key.code {
            KeyCode::Esc => Some(Action::CloseLauncher),
            KeyCode::Up => Some(Action::MoveLauncherUp),
            KeyCode::Down => Some(Action::MoveLauncherDown),
            KeyCode::Enter => Some(Action::ExecuteLauncherSelection),
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

pub fn action_for_mouse(
    mouse: MouseEvent,
    state: &AppState,
    geometry: &UiGeometry,
) -> Option<Action> {
    if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
        return None;
    }
    let position = Position::new(mouse.column, mouse.row);
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

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn launcher_and_terminal_shortcuts_are_translated() {
        assert_eq!(
            action_for_key(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE), false),
            Some(Action::ToggleLauncher)
        );
        assert_eq!(
            action_for_key(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE), false),
            Some(Action::OpenApplication(ApplicationKind::Terminal))
        );
    }

    #[test]
    fn launcher_navigation_does_not_leak_to_desktop_actions() {
        assert_eq!(
            action_for_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE), true),
            Some(Action::MoveLauncherDown)
        );
        assert_eq!(
            action_for_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE), true),
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
}
