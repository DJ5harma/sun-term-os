use ratatui::{Frame, layout::Rect};

use crate::app::AppState;

use super::{home_screen, interaction::InteractionMap, windows};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, interactions: &mut InteractionMap) {
    if let Some(window) = state.visible_focus_window() {
        windows::render(frame, area, state, window, interactions);
        return;
    }
    if let Some(mode) = state.home_screen_mode() {
        home_screen::render(frame, area, state, mode, interactions);
    }
}
