use crate::{
    actions::AsyncAction,
    app::{AppState, Loadable},
};

use super::Effect;

pub(super) fn reduce(state: &mut AppState, action: AsyncAction) -> Vec<Effect> {
    match action {
        AsyncAction::SystemInfoReady(result) => {
            let listing = match result {
                Ok(snapshot) => Loadable::Ready(snapshot),
                Err(error) => Loadable::Failed(error),
            };
            state.system = listing.clone();
            if matches!(&listing, Loadable::Ready(_)) {
                state.capabilities_refreshed_at = Some(current_unix_secs());
            }
            for view in state.system_info_views.values_mut() {
                *view = listing.clone();
            }
        }
        AsyncAction::ProcessesReady(result) => {
            let listing = match result {
                Ok(processes) => Loadable::Ready(processes),
                Err(error) => Loadable::Failed(error),
            };
            state.processes = listing.clone();
            let refreshed = matches!(&listing, Loadable::Ready(_));
            let refreshed_at = refreshed.then_some(current_unix_secs());
            for manager in state.process_managers.values_mut() {
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
        AsyncAction::DirectoryReady(window_id, result) => {
            if let Some(manager) = state.file_managers.get_mut(&window_id) {
                manager.listing = match result {
                    Ok(listing) => {
                        manager.current_path = listing.path.clone();
                        Loadable::Ready(crate::app::file_manager::apply_sorted_listing(
                            listing,
                            manager.sort,
                        ))
                    }
                    Err(error) => Loadable::Failed(error),
                };
                manager.clamp_selection();
            }
        }
    }
    Vec::new()
}

fn current_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
