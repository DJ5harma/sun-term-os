use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;

use crate::{
    actions::Action,
    app::{AppState, ApplicationKind, SortColumn},
    ui::{geometry::UiGeometry, hit_map::HitMap},
};

use super::shell_shortcuts::{consumes_for_shell, resolve as resolve_shell};

/// New window shortcuts when the focused app is not a terminal (typing must not be stolen).
fn quick_launch(key: KeyEvent) -> Option<Action> {
    if key.modifiers != KeyModifiers::NONE {
        return None;
    }
    match key.code {
        KeyCode::Char('t') => Some(Action::OpenApplication(ApplicationKind::Terminal)),
        KeyCode::Char('f') => Some(Action::OpenApplication(ApplicationKind::FileManager)),
        _ => None,
    }
}

pub fn action_for_key(
    key: KeyEvent,
    launcher_open: bool,
    terminal_focused: bool,
    file_manager_focused: bool,
) -> Option<Action> {
    if let Some(action) = resolve_shell(key) {
        return Some(action);
    }

    if launcher_open {
        return match key.code {
            KeyCode::Esc => Some(Action::CloseLauncher),
            KeyCode::Up => Some(Action::MoveLauncherUp),
            KeyCode::Down => Some(Action::MoveLauncherDown),
            KeyCode::Enter => Some(Action::ExecuteLauncherSelection),
            _ => None,
        };
    }

    if file_manager_focused {
        return match key {
            KeyEvent {
                code: KeyCode::Up, ..
            } => Some(Action::MoveFileSelection(-1)),
            KeyEvent {
                code: KeyCode::Down,
                ..
            } => Some(Action::MoveFileSelection(1)),
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => Some(Action::OpenSelectedEntry),
            KeyEvent {
                code: KeyCode::Backspace,
                ..
            }
            | KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::ALT,
                ..
            } => Some(Action::FileManagerGoBack),
            KeyEvent {
                code: KeyCode::Char('u'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(Action::FileManagerGoUp),
            KeyEvent {
                code: KeyCode::Char('g'),
                modifiers: KeyModifiers::NONE,
                ..
            }
            | KeyEvent {
                code: KeyCode::Char('~'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(Action::FileManagerGoHome),
            KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(Action::FileManagerTogglePane),
            KeyEvent {
                code: KeyCode::PageUp,
                ..
            } => Some(Action::FileManagerPageScroll(-1)),
            KeyEvent {
                code: KeyCode::PageDown,
                ..
            } => Some(Action::FileManagerPageScroll(1)),
            KeyEvent {
                code: KeyCode::Char('h'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(Action::ToggleFileManagerHidden),
            KeyEvent {
                code: KeyCode::Char('r'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(Action::ReloadFileManager),
            KeyEvent {
                code: KeyCode::Char('1'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(Action::FileManagerSetSort(SortColumn::Name)),
            KeyEvent {
                code: KeyCode::Char('2'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(Action::FileManagerSetSort(SortColumn::Size)),
            KeyEvent {
                code: KeyCode::Char('3'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(Action::FileManagerSetSort(SortColumn::Modified)),
            _ => quick_launch(key),
        };
    }

    if terminal_focused {
        return None;
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
        other => quick_launch(other),
    }
}

pub fn terminal_input(key: KeyEvent) -> Option<Vec<u8>> {
    if consumes_for_shell(key) {
        return None;
    }
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
    hit_map: &HitMap,
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
        return hit_map.hit(mouse.column, mouse.row);
    }
    let file_manager_focused = state
        .focused_window()
        .is_some_and(|window| window.application == ApplicationKind::FileManager);
    if file_manager_focused {
        if mouse.kind == MouseEventKind::ScrollUp {
            return Some(Action::FileManagerPageScroll(-1));
        }
        if mouse.kind == MouseEventKind::ScrollDown {
            return Some(Action::FileManagerPageScroll(1));
        }
    }
    if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
        return None;
    }
    hit_map.hit(mouse.column, mouse.row)
}

pub fn terminal_mouse(mouse: MouseEvent, geometry: &UiGeometry) -> Option<Vec<u8>> {
    let position = Position::new(mouse.column, mouse.row);
    if geometry.top_bar.contains(position) || geometry.bottom_bar.contains(position) {
        return None;
    }
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

        assert_eq!(action_for_key(key, false, true, false), None);
        assert_eq!(terminal_input(key), Some(vec![3]));
    }

    #[test]
    fn launcher_and_terminal_shortcuts_are_translated() {
        assert_eq!(
            action_for_key(
                KeyEvent::new(
                    KeyCode::Char('p'),
                    KeyModifiers::CONTROL | KeyModifiers::SHIFT,
                ),
                false,
                false,
                false,
            ),
            Some(Action::ToggleLauncher)
        );
        assert_eq!(
            action_for_key(
                KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE),
                false,
                false,
                false
            ),
            Some(Action::OpenApplication(ApplicationKind::Terminal))
        );
        assert_eq!(
            action_for_key(
                KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE),
                false,
                false,
                false
            ),
            Some(Action::OpenApplication(ApplicationKind::FileManager))
        );
    }

    #[test]
    fn launcher_navigation_does_not_leak_to_desktop_actions() {
        assert_eq!(
            action_for_key(
                KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
                true,
                false,
                false
            ),
            Some(Action::MoveLauncherDown)
        );
        assert_eq!(
            action_for_key(
                KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
                true,
                false,
                false
            ),
            None
        );
    }

    #[test]
    fn launcher_chord_works_while_file_manager_is_focused() {
        let chord = KeyEvent::new(
            KeyCode::Char('p'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );
        assert_eq!(
            action_for_key(chord, false, false, true),
            Some(Action::ToggleLauncher)
        );
    }

    #[test]
    fn bare_p_reaches_terminal_when_focused() {
        assert_eq!(
            action_for_key(
                KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE),
                false,
                true,
                false,
            ),
            None
        );
        assert!(terminal_input(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE)).is_some());
    }

    #[test]
    fn file_manager_navigation_is_translated_when_focused() {
        assert_eq!(
            action_for_key(
                KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
                false,
                false,
                true
            ),
            Some(Action::MoveFileSelection(1))
        );
        assert_eq!(
            action_for_key(
                KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
                false,
                false,
                true
            ),
            Some(Action::OpenSelectedEntry)
        );
    }

    #[test]
    fn f_keys_switch_workspaces_from_any_app() {
        assert_eq!(
            action_for_key(
                KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE),
                false,
                true,
                false
            ),
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
        let mut hits = HitMap::default();
        let toggle_area = Rect::new(0, 0, 18, 1);
        hits.register(toggle_area, Action::ToggleLauncher);
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 1,
            row: 0,
            modifiers: KeyModifiers::NONE,
        };
        assert_eq!(
            action_for_mouse(mouse, &state, &geometry, &hits),
            Some(Action::ToggleLauncher)
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

    #[test]
    fn terminal_mouse_ignores_top_bar_clicks() {
        let state = AppState::default();
        let geometry = crate::ui::geometry::calculate(Rect::new(0, 0, 120, 40), &state);
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: geometry.top_bar.x + 1,
            row: geometry.top_bar.y + 1,
            modifiers: KeyModifiers::NONE,
        };
        assert!(terminal_mouse(mouse, &geometry).is_none());
    }
}
