use ratatui::{Frame, layout::Rect};

use crate::{
    app::{Loadable, effects::Effect, state::AppState},
    domain::{ApplicationKind, Window, WindowId},
    ui::interaction::InteractionMap,
};

use super::{BuiltInApp, shell_keys};

pub const APP: BuiltInApp = BuiltInApp {
    kind: ApplicationKind::SystemInfo,
    title: "System Information",
    short_title: "Sys",
    description: "Inspect this machine",
    quick_launch_key: Some('s'),
    on_open,
    on_close,
    on_open_window: None,
    render,
    dispatch_key: shell_keys::dispatch_key,
    desktop_scroll: None,
    palette_extras: None,
};

pub fn on_open(state: &mut AppState, window_id: WindowId) -> Vec<Effect> {
    let machine_id = state
        .window_machine_id(window_id)
        .unwrap_or(crate::machine::MachineId::Local);
    let listing = state.system_for(&machine_id);
    state.init_system_info_view(window_id, listing);
    Vec::new()
}

pub fn on_close(state: &mut AppState, window_id: WindowId) -> Vec<Effect> {
    state.remove_system_info_view(window_id);
    Vec::new()
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    window: &Window,
    _interactions: &mut InteractionMap,
) {
    let listing = state
        .system_info_view(window.id)
        .cloned()
        .unwrap_or(Loadable::Loading);
    crate::ui::system_info::render(
        frame,
        area,
        state,
        window.id,
        &listing,
        state.capabilities_refreshed_at,
    );
}
