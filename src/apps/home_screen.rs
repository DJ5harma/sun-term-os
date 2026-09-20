use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    actions::{Action, HomeScreenAction},
    app::AppState,
};

use super::shell_keys::quick_launch_action;

pub fn dispatch_key(key: KeyEvent, state: &AppState) -> Option<Action> {
    if !state.shows_home_screen() {
        return None;
    }
    if let Some(action) = quick_launch_action(key) {
        return Some(action);
    }
    match key {
        KeyEvent {
            code: KeyCode::Up, ..
        } => Some(Action::HomeScreen(HomeScreenAction::MoveRow(-1))),
        KeyEvent {
            code: KeyCode::Down,
            ..
        } => Some(Action::HomeScreen(HomeScreenAction::MoveRow(1))),
        KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::HomeScreen(HomeScreenAction::MoveSelection(-1))),
        KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::HomeScreen(HomeScreenAction::MoveSelection(1))),
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => Some(Action::HomeScreen(HomeScreenAction::ActivateSelected)),
        _ => None,
    }
}

pub fn desktop_scroll_action(delta: i32) -> Action {
    Action::HomeScreen(HomeScreenAction::PageScroll(delta))
}
