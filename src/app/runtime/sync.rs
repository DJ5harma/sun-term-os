use crate::{
    app::{
        ApplicationKind, home_screen::entries, palette, process_manager,
        state::{AppState, Loadable},
    },
    ui,
};

fn list_rows_in_window(
    geometry: &ui::geometry::UiGeometry,
    state: &AppState,
    kind: ApplicationKind,
) -> Option<usize> {
    let window = state
        .focused_window()
        .filter(|window| window.application == kind)?;
    let inner = ui::windows::content_inner(geometry.desktop, window, state);
    Some(inner.height.saturating_sub(4).max(1) as usize)
}

pub fn sync_home_screen_layout(state: &mut AppState, geometry: &ui::geometry::UiGeometry) {
    if !state.shows_home_screen() {
        return;
    }
    let mode = state
        .home_screen_mode()
        .unwrap_or(crate::app::home_screen::HomeScreenMode::EmptyWorkspace);
    // mode is always Some when shows_home_screen(); unwrap_or is a fallback only.
    let list = entries(&state.config);
    let layout = ui::home_screen::layout(geometry.desktop, list.len(), mode);
    let home = state.home_screen_mut();
    home.columns = layout.columns;
    home.tile_rows_visible = layout.tile_rows_visible;
    home.clamp_selection(list.len(), layout.total_tile_rows);
}

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

pub fn sync_launcher_visible_rows(state: &mut AppState, geometry: &ui::geometry::UiGeometry) {
    let Some(rows) = list_rows_in_window(geometry, state, ApplicationKind::Launcher) else {
        return;
    };
    if let Some(window) = state.focused_window()
        && let Some(view) = state.launcher_view_mut(window.id)
    {
        view.visible_rows = rows;
        let count = match &view.listing {
            Loadable::Ready(entries) => {
                let filter = view.filter.to_lowercase();
                entries
                    .iter()
                    .filter(|entry| {
                        filter.is_empty()
                            || format!("{} {}", entry.name, entry.detail)
                                .to_lowercase()
                                .contains(&filter)
                    })
                    .count()
            }
            _ => 0,
        };
        view.clamp_selection(count);
    }
}

pub fn sync_services_visible_rows(state: &mut AppState, geometry: &ui::geometry::UiGeometry) {
    let Some(rows) = list_rows_in_window(geometry, state, ApplicationKind::Services) else {
        return;
    };
    if let Some(window) = state.focused_window()
        && let Some(view) = state.services_view_mut(window.id)
    {
        view.visible_rows = rows;
        let count = match &view.listing {
            Loadable::Ready(services) => {
                let filter = view.filter.to_lowercase();
                services
                    .iter()
                    .filter(|service| {
                        filter.is_empty()
                            || service.name.to_lowercase().contains(&filter)
                            || service.active_state.to_lowercase().contains(&filter)
                    })
                    .count()
            }
            _ => 0,
        };
        view.clamp_selection(count);
    }
}

pub fn sync_text_viewer_visible_rows(state: &mut AppState, geometry: &ui::geometry::UiGeometry) {
    let Some(rows) = list_rows_in_window(geometry, state, ApplicationKind::TextViewer) else {
        return;
    };
    if let Some(window) = state.focused_window()
        && let Some(view) = state.text_viewer_mut(window.id)
    {
        view.visible_rows = rows;
        let line_count = match &view.content {
            Loadable::Ready(text) => text.lines().count(),
            _ => 0,
        };
        view.clamp_scroll(line_count);
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
