use crate::{
    actions::AsyncAction,
    app::{AppState, Loadable},
    machine::MachineId,
};

use super::Effect;

pub(super) fn reduce(state: &mut AppState, action: AsyncAction) -> Vec<Effect> {
    match action {
        AsyncAction::SystemInfoReady(machine_id, result) => {
            let listing = match result {
                Ok(snapshot) => Loadable::Ready(snapshot),
                Err(error) => Loadable::Failed(error),
            };
            state.systems.insert(machine_id.clone(), listing.clone());
            if matches!(&listing, Loadable::Ready(_)) {
                state.capabilities_refreshed_at = Some(current_unix_secs());
            }
            let targets =
                matching_windows(state, &machine_id, state.system_info_views.keys().copied());
            for window_id in targets {
                if let Some(view) = state.system_info_views.get_mut(&window_id) {
                    *view = listing.clone();
                }
            }
        }
        AsyncAction::ProcessesReady(machine_id, result) => {
            let listing = match result {
                Ok(processes) => Loadable::Ready(processes),
                Err(error) => Loadable::Failed(error),
            };
            state
                .process_lists
                .insert(machine_id.clone(), listing.clone());
            let refreshed = matches!(&listing, Loadable::Ready(_));
            let refreshed_at = refreshed.then_some(current_unix_secs());
            let targets =
                matching_windows(state, &machine_id, state.process_managers.keys().copied());
            for window_id in targets {
                if let Some(manager) = state.process_managers.get_mut(&window_id) {
                    manager.listing = listing.clone();
                    if refreshed_at.is_some() {
                        manager.last_refreshed_at = refreshed_at;
                    }
                    if let Loadable::Ready(processes) = &manager.listing {
                        let count = crate::app::process_manager::matching_indices(
                            processes,
                            &manager.filter,
                            manager.sort,
                        )
                        .len();
                        manager.clamp_selection(count);
                    }
                }
            }
        }
        AsyncAction::DirectoryReady(window_id, result) => {
            let sort = state
                .file_managers
                .get(&window_id)
                .map(|manager| manager.sort)
                .unwrap_or_default();
            let listing = match result {
                Ok(listing) => {
                    let sorted = crate::app::file_manager::apply_sorted_listing(listing, sort);
                    Loadable::Ready(sorted)
                }
                Err(error) => Loadable::Failed(crate::app::offline::enrich_load_error(
                    state, window_id, &error,
                )),
            };
            if let Some(manager) = state.file_managers.get_mut(&window_id) {
                if let Loadable::Ready(sorted) = &listing {
                    manager.current_path = sorted.path.clone();
                }
                manager.listing = listing;
                manager.clamp_selection();
            }
        }
        AsyncAction::LauncherReady(machine_id, result) => {
            let listing = match result {
                Ok(entries) => Loadable::Ready(entries),
                Err(error) => Loadable::Failed(error),
            };
            let targets =
                matching_windows(state, &machine_id, state.launcher_views.keys().copied());
            for window_id in targets {
                if let Some(view) = state.launcher_views.get_mut(&window_id) {
                    view.listing = listing.clone();
                }
            }
        }
        AsyncAction::TextFileReady(window_id, result) => {
            if let Some(view) = state.text_viewers.get_mut(&window_id) {
                view.content = match result {
                    Ok(text) => Loadable::Ready(text),
                    Err(error) => Loadable::Failed(error),
                };
            }
        }
        AsyncAction::ServicesReady(machine_id, result) => {
            let listing = match result {
                Ok(services) => Loadable::Ready(services),
                Err(error) => Loadable::Failed(error),
            };
            let targets =
                matching_windows(state, &machine_id, state.services_views.keys().copied());
            for window_id in targets {
                if let Some(view) = state.services_views.get_mut(&window_id) {
                    view.listing = listing.clone();
                }
            }
        }
    }
    Vec::new()
}

fn matching_windows(
    state: &AppState,
    machine_id: &MachineId,
    window_ids: impl IntoIterator<Item = u64>,
) -> Vec<u64> {
    window_ids
        .into_iter()
        .filter(|window_id| state.window_machine_id(*window_id).as_ref() == Some(machine_id))
        .collect()
}

fn current_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
