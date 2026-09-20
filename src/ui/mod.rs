pub mod geometry;
pub mod hit_map;

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
use hit_map::HitMap;

pub fn render(frame: &mut Frame, state: &AppState, geometry: &UiGeometry, hits: &mut HitMap) {
    frame.render_widget(Block::default().style(theme::base()), frame.area());
    top_bar::render(frame, geometry.top_bar, state, hits);
    desktop::render(frame, geometry.desktop, state, hits);
    bottom_bar::render(frame, geometry.bottom_bar, state, hits);
    if state.launcher_open {
        launcher::render(frame, geometry.launcher, state, hits);
    }
}
