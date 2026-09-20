use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use crate::{
    actions::{Action, FileManagerAction, PaletteAction, ShellAction},
    app::{AppState, ApplicationKind},
    ui::interaction::InteractionMap,
};

use super::test_helpers::{action_for_key, terminal_input};

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
        Some(Action::Palette(PaletteAction::ToggleLauncher))
    );
    assert_eq!(
        action_for_key(
            KeyEvent::new(
                KeyCode::Char('p'),
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
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
        Some(Action::Shell(ShellAction::OpenApplication(
            ApplicationKind::Terminal
        )))
    );
    assert_eq!(
        action_for_key(
            KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE),
            false,
            false,
            false
        ),
        Some(Action::Shell(ShellAction::OpenApplication(
            ApplicationKind::FileManager
        )))
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
        Some(Action::Palette(PaletteAction::MoveLauncherDown))
    );
    assert_eq!(
        action_for_key(
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
            true,
            false,
            false
        ),
        Some(Action::Palette(PaletteAction::PaletteQueryPush('q')))
    );
}

#[test]
fn launcher_chord_works_while_file_manager_is_focused() {
    let chord = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::ALT);
    assert_eq!(
        action_for_key(chord, false, false, true),
        Some(Action::Palette(PaletteAction::ToggleLauncher))
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
        Some(Action::Shell(ShellAction::BeginWindowPick))
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
        Some(Action::FileManager(FileManagerAction::MoveFileSelection(1)))
    );
    assert_eq!(
        action_for_key(
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            false,
            false,
            true
        ),
        Some(Action::FileManager(FileManagerAction::OpenSelectedEntry))
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
        Some(Action::Shell(ShellAction::SwitchWorkspace(1)))
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
        vec![Action::Palette(PaletteAction::ToggleLauncher)]
    );
}
