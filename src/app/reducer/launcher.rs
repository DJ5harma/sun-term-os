use crate::{
    actions::LauncherAction,
    app::{AppState, Loadable, effects::LauncherEffect},
    domain::ApplicationKind,
    machine::applications::ApplicationEntry,
};

use super::Effect;

pub(super) fn reduce(state: &mut AppState, action: LauncherAction) -> Vec<Effect> {
    let window_id = focused_launcher_window(state);
    match action {
        LauncherAction::MoveSelection(offset) => {
            if let Some(id) = window_id {
                let count = filtered_entries(state, id).len();
                if let Some(view) = state.launcher_view_mut(id) {
                    let next = view.selected_index as i32 + offset;
                    view.selected_index = next.clamp(0, count as i32 - 1).max(0) as usize;
                    view.clamp_selection(count);
                }
            }
        }
        LauncherAction::LauncherPageScroll(pages) => {
            if let Some(id) = window_id {
                let count = filtered_entries(state, id).len();
                if let Some(view) = state.launcher_view_mut(id) {
                    let delta = pages * view.visible_rows.max(1) as i32;
                    let next = view.selected_index as i32 + delta;
                    view.selected_index = next.clamp(0, count as i32 - 1).max(0) as usize;
                    view.clamp_selection(count);
                }
            }
        }
        LauncherAction::LauncherFilterBegin => {
            if let Some(id) = window_id
                && let Some(view) = state.launcher_view_mut(id)
            {
                view.filter_active = true;
            }
        }
        LauncherAction::LauncherFilterPush(ch) => {
            if let Some(id) = window_id
                && let Some(view) = state.launcher_view_mut(id)
            {
                view.filter.push(ch);
            }
        }
        LauncherAction::LauncherFilterBackspace => {
            if let Some(id) = window_id
                && let Some(view) = state.launcher_view_mut(id)
            {
                view.filter.pop();
            }
        }
        LauncherAction::LauncherFilterEnd => {
            if let Some(id) = window_id
                && let Some(view) = state.launcher_view_mut(id)
            {
                view.filter_active = false;
                view.selected_index = 0;
                view.scroll_offset = 0;
            }
        }
        LauncherAction::LaunchSelected => {
            if let Some(id) = window_id {
                let entries = filtered_entries(state, id);
                if let Some(entry) = entries.get(
                    state
                        .launcher_view(id)
                        .map(|view| view.selected_index)
                        .unwrap_or(0),
                ) {
                    return vec![Effect::Launcher(LauncherEffect::Launch(entry.clone()))];
                }
            }
        }
        LauncherAction::ToggleFavorite => {
            if let Some(id) = window_id {
                let entries = filtered_entries(state, id);
                let index = state
                    .launcher_view(id)
                    .map(|view| view.selected_index)
                    .unwrap_or(0);
                if let Some(entry) = entries.get(index) {
                    let entry_id = entry.id.clone();
                    let favorites = &mut state.config.launcher.favorites;
                    if let Some(position) = favorites.iter().position(|id| id == &entry_id) {
                        favorites.remove(position);
                        state.status = format!("Unpinned {}", entry.name);
                    } else {
                        favorites.push(entry_id);
                        state.status = format!("Pinned {}", entry.name);
                    }
                    let machine_id = state
                        .window_machine_id(id)
                        .unwrap_or(crate::machine::MachineId::Local);
                    return vec![
                        Effect::PersistConfig,
                        Effect::Launcher(LauncherEffect::Discover(machine_id)),
                    ];
                }
            }
        }
    }
    Vec::new()
}

fn focused_launcher_window(state: &AppState) -> Option<u64> {
    state
        .focused_window()
        .filter(|window| window.application == ApplicationKind::Launcher)
        .map(|window| window.id)
}

fn filtered_entries(state: &AppState, window_id: u64) -> Vec<ApplicationEntry> {
    let view = state.launcher_view(window_id);
    let listing = view
        .map(|view| view.listing.clone())
        .unwrap_or(Loadable::Loading);
    let filter = view
        .map(|view| view.filter.to_lowercase())
        .unwrap_or_default();
    match listing {
        Loadable::Ready(entries) => entries
            .into_iter()
            .filter(|entry| {
                if filter.is_empty() {
                    true
                } else {
                    format!("{} {}", entry.name, entry.detail)
                        .to_lowercase()
                        .contains(&filter)
                }
            })
            .collect(),
        _ => Vec::new(),
    }
}
