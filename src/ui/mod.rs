pub mod geometry;

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

pub fn render(frame: &mut Frame, state: &AppState, geometry: &UiGeometry) {
    frame.render_widget(Block::default().style(theme::base()), frame.area());
    top_bar::render(frame, geometry.top_bar, state, geometry);
    desktop::render(frame, geometry.desktop, state);
    bottom_bar::render(frame, geometry.bottom_bar, state, geometry);
    if state.launcher_open {
        launcher::render(frame, geometry.launcher, state);
    }
}
