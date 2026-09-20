mod keybindings;

use crossterm::event::{KeyEvent, MouseEvent};

use crate::{actions::Action, app::AppState, ui::geometry::UiGeometry};

pub fn action_for_key(key: KeyEvent, launcher_open: bool) -> Option<Action> {
    keybindings::action_for_key(key, launcher_open)
}

pub fn action_for_mouse(
    mouse: MouseEvent,
    state: &AppState,
    geometry: &UiGeometry,
) -> Option<Action> {
    keybindings::action_for_mouse(mouse, state, geometry)
}
