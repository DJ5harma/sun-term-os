pub mod geometry;
pub mod interaction;

pub mod app_launcher;
mod bottom_bar;
mod desktop;
pub mod file_manager;
pub mod home_screen;
pub mod launcher;
pub mod machines;
pub(crate) mod processes;
pub mod services;
pub mod settings;
pub mod system_info;
pub mod terminal;
pub mod text_viewer;
pub(crate) mod theme;
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
