mod keybindings;

use crossterm::event::{KeyEvent, MouseEvent};

use crate::{actions::Action, app::AppState, ui::geometry::UiGeometry};

pub fn action_for_key(
    key: KeyEvent,
    launcher_open: bool,
    terminal_focused: bool,
) -> Option<Action> {
    keybindings::action_for_key(key, launcher_open, terminal_focused)
}

pub fn terminal_input(key: KeyEvent) -> Option<Vec<u8>> {
    keybindings::terminal_input(key)
}

pub fn terminal_mouse(mouse: MouseEvent, geometry: &UiGeometry) -> Option<Vec<u8>> {
    keybindings::terminal_mouse(mouse, geometry)
}

pub fn action_for_mouse(
    mouse: MouseEvent,
    state: &AppState,
    geometry: &UiGeometry,
) -> Option<Action> {
    keybindings::action_for_mouse(mouse, state, geometry)
}
