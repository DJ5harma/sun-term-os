pub mod geometry;
pub mod interaction;

mod bottom_bar;
mod desktop;
pub mod file_manager;
mod launcher;
mod theme;
mod top_bar;
pub(crate) mod windows;

use ratatui::{Frame, widgets::Block};

use crate::app::AppState;

use geometry::UiGeometry;
use interaction::InteractionMap;

pub fn render(
    frame: &mut Frame,
    state: &AppState,
    geometry: &UiGeometry,
    interactions: &mut InteractionMap,
) {
    frame.render_widget(Block::default().style(theme::base()), frame.area());
    top_bar::render(frame, geometry.top_bar, state);
    desktop::render(frame, geometry.desktop, state, interactions);
    bottom_bar::render(frame, geometry.bottom_bar, state);
    if state.launcher_open {
        launcher::render(frame, geometry.launcher, state, interactions);
    }
}
