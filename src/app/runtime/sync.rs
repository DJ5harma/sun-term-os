use crate::{
    app::{
        ApplicationKind, palette, process_manager,
        state::{AppState, Loadable},
    },
    ui,
};

pub fn sync_palette_selection(state: &mut AppState, geometry: &ui::geometry::UiGeometry) {
    if !state.launcher_open {
        return;
    }
    state.launcher_visible_rows = ui::launcher::list_visible_rows(geometry.launcher);
    let count = palette::filtered_entries(state).len();
    palette::clamp_palette_selection(
        &mut state.launcher_selection,
        &mut state.launcher_scroll_offset,
        state.launcher_visible_rows.max(1),
        count,
    );
}

pub fn sync_process_manager_visible_rows(
    state: &mut AppState,
    geometry: &ui::geometry::UiGeometry,
) {
    let Some(window) = state
        .focused_window()
        .filter(|window| window.application == ApplicationKind::Processes)
    else {
        return;
    };
    let window_inner = ui::windows::content_inner(geometry.desktop, window, state);
    let rows = window_inner.height.saturating_sub(4) as usize;
    let match_count = state
        .process_manager(window.id)
        .map(|manager| match &manager.listing {
            Loadable::Ready(processes) => {
                process_manager::matching_indices(processes, &manager.filter, manager.sort).len()
            }
            _ => 0,
        })
        .unwrap_or(0);
    if let Some(manager) = state.process_manager_mut(window.id) {
        manager.visible_rows = rows.max(1);
        manager.clamp_selection(match_count);
    }
}

pub fn sync_file_manager_visible_rows(state: &mut AppState, geometry: &ui::geometry::UiGeometry) {
    let Some(window) = state
        .focused_window()
        .filter(|window| window.application == ApplicationKind::FileManager)
    else {
        return;
    };
    let window_inner = ui::windows::content_inner(geometry.desktop, window, state);
    let layout = ui::file_manager::layout(window_inner);
    if let Some(manager) = state.file_manager_mut(window.id) {
        ui::file_manager::sync_visible_rows(manager, layout.list_rows);
        manager.clamp_selection();
    }
}
