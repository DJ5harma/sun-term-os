use crate::{
    actions::HomeScreenAction,
    app::{AppState, home_screen::entries},
};

use super::{Effect, open_application, open_application_effects};

pub(super) fn reduce(state: &mut AppState, action: HomeScreenAction) -> Vec<Effect> {
    if !state.shows_home_screen() {
        return Vec::new();
    }
    let count = entries(&state.config).len();
    let columns = state.home_screen().columns.max(1);
    if matches!(action, HomeScreenAction::ActivateSelected) {
        let selected = state.home_screen().selected;
        if let Some(entry) = entries(&state.config).get(selected) {
            let kind = entry.kind;
            let window_id = open_application(state, kind);
            return open_application_effects(state, kind, window_id);
        }
        return Vec::new();
    }
    let home = state.home_screen_mut();
    match action {
        HomeScreenAction::MoveSelection(delta) => {
            let next = home.selected as i32 + delta;
            home.selected = next.clamp(0, count as i32 - 1).max(0) as usize;
            let total_rows = tile_row_count(count, columns);
            home.clamp_selection(count, total_rows);
        }
        HomeScreenAction::MoveRow(delta) => {
            let row = home.selected / columns;
            let col = home.selected % columns;
            let total_rows = tile_row_count(count, columns);
            let next_row = (row as i32 + delta).clamp(0, total_rows as i32 - 1) as usize;
            let index = next_row * columns + col;
            home.selected = index.min(count.saturating_sub(1));
            home.clamp_selection(count, total_rows);
        }
        HomeScreenAction::PageScroll(pages) => {
            let total_rows = tile_row_count(count, columns);
            let delta = pages * home.tile_rows_visible.max(1) as i32;
            let next = home.scroll_row as i32 + delta;
            home.scroll_row = next.clamp(0, total_rows as i32 - 1).max(0) as usize;
            home.clamp_selection(count, total_rows);
        }
        HomeScreenAction::ActivateSelected => {}
    }
    Vec::new()
}

fn tile_row_count(entry_count: usize, columns: usize) -> usize {
    if entry_count == 0 {
        0
    } else {
        entry_count.div_ceil(columns.max(1))
    }
}
