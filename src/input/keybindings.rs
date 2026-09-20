use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    actions::Action,
    app::{ApplicationKind, SortColumn},
};

/// New window shortcuts when the focused app is not a terminal (typing must not be stolen).
fn quick_launch(key: KeyEvent) -> Option<Action> {
    if key.modifiers != KeyModifiers::NONE {
        return None;
    }
    match key.code {
        KeyCode::Char('t') => Some(Action::OpenApplication(ApplicationKind::Terminal)),
        KeyCode::Char('f') => Some(Action::OpenApplication(ApplicationKind::FileManager)),
        KeyCode::Char('p') => Some(Action::OpenApplication(ApplicationKind::Processes)),
        KeyCode::Char('s') => Some(Action::OpenApplication(ApplicationKind::SystemInfo)),
        _ => None,
    }
}

pub fn file_manager_key(key: KeyEvent) -> Option<Action> {
    match key {
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
        KeyEvent {
            code: KeyCode::Char('o'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::FileManagerOpenInTerminal),
        _ => quick_launch(key),
    }
}

pub fn process_manager_key(key: KeyEvent) -> Option<Action> {
    match key {
        KeyEvent {
            code: KeyCode::Up, ..
        } => Some(Action::MoveProcessSelection(-1)),
        KeyEvent {
            code: KeyCode::Down,
            ..
        } => Some(Action::MoveProcessSelection(1)),
        KeyEvent {
            code: KeyCode::PageUp,
            ..
        } => Some(Action::ProcessPageScroll(-1)),
        KeyEvent {
            code: KeyCode::PageDown,
            ..
        } => Some(Action::ProcessPageScroll(1)),
        KeyEvent {
            code: KeyCode::Char('/'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::ProcessFilterBegin),
        KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::ProcessKillSelected),
        KeyEvent {
            code: KeyCode::Char('r'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::Refresh),
        other => quick_launch(other),
    }
}

pub fn process_filter_key(key: KeyEvent) -> Option<Action> {
    match key {
        KeyEvent {
            code: KeyCode::Esc, ..
        } => Some(Action::ProcessFilterEnd),
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => Some(Action::ProcessFilterEnd),
        KeyEvent {
            code: KeyCode::Backspace,
            ..
        } => Some(Action::ProcessFilterBackspace),
        KeyEvent {
            code: KeyCode::Char(character),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::ProcessFilterPush(character)),
        _ => None,
    }
}

pub fn desktop_key(key: KeyEvent) -> Option<Action> {
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
            code: KeyCode::Char('r'),
            ..
        } => Some(Action::Refresh),
        other => quick_launch(other),
    }
}

#[cfg(test)]
pub fn action_for_key(
    key: KeyEvent,
    launcher_open: bool,
    terminal_focused: bool,
    file_manager_focused: bool,
) -> Option<Action> {
    use super::router::{FocusContext, KeyDispatch, KeyInputContext, dispatch_key};
    let focus = if launcher_open {
        FocusContext::Launcher
    } else if terminal_focused {
        FocusContext::Window(ApplicationKind::Terminal)
    } else if file_manager_focused {
        FocusContext::Window(ApplicationKind::FileManager)
    } else {
        FocusContext::Chrome
    };
    match dispatch_key(
        key,
        &KeyInputContext {
            focus,
            window_pick_mode: false,
            process_filter_active: false,
        },
    ) {
        KeyDispatch::Action(action) => Some(action),
        KeyDispatch::Terminal(_) | KeyDispatch::Consumed => None,
    }
}

#[cfg(test)]
pub fn terminal_input(key: KeyEvent) -> Option<Vec<u8>> {
    crate::input::terminal_encode::encode_key(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
    use ratatui::layout::Rect;

    use crate::app::AppState;
    use crate::ui::interaction::InteractionMap;

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
                KeyEvent::new(KeyCode::Char('p'), KeyModifiers::ALT),
                false,
                false,
                false,
            ),
            Some(Action::ToggleLauncher)
        );
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
            None
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
        let chord = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::ALT);
        assert_eq!(
            action_for_key(chord, false, false, true),
            Some(Action::ToggleLauncher)
        );
    }

    #[test]
    fn ctrl_g_begins_window_pick_from_file_manager() {
        assert_eq!(
            action_for_key(
                KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL),
                false,
                false,
                true,
            ),
            Some(Action::BeginWindowPick)
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
    fn launcher_clicks_use_bottom_bar_shell_geometry() {
        let state = AppState::default();
        let geometry = crate::ui::geometry::calculate(Rect::new(0, 0, 120, 40), &state);
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: geometry.bottom_bar.launcher.x + 2,
            row: geometry.bottom_bar.launcher.y,
            modifiers: KeyModifiers::NONE,
        };
        assert_eq!(
            crate::input::actions_for_mouse(
                mouse,
                &state,
                &geometry,
                &InteractionMap::default(),
                &mut Default::default()
            ),
            vec![Action::ToggleLauncher]
        );
    }
}
