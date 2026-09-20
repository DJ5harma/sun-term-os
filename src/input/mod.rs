mod bindings;
mod keybindings;
mod mouse_click;
pub mod normalize;
mod pointer;
mod router;
pub mod terminal_encode;

pub use bindings::{LAUNCHER_SHORTCUT_HINT, WINDOW_FOCUS_HINT, WORKSPACE_HINT};
pub use mouse_click::DoubleClickState;

use crossterm::event::{KeyEvent, MouseEvent};

use crate::{
    actions::Action,
    app::AppState,
    ui::{geometry::UiGeometry, interaction::InteractionMap},
};

pub use router::{FocusContext, KeyDispatch, KeyInputContext, dispatch_key};

pub fn handle_key(key: KeyEvent, context: &KeyInputContext) -> KeyDispatch {
    dispatch_key(key, context)
}

pub fn actions_for_mouse(
    mouse: MouseEvent,
    state: &AppState,
    geometry: &UiGeometry,
    interactions: &InteractionMap,
    double_click: &mut DoubleClickState,
) -> Vec<Action> {
    pointer::dispatch_pointer(mouse, state, geometry, interactions, double_click).actions
}
