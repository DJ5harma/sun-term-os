use crate::app::AppState;
use crate::domain::WindowId;
use crate::machine::{MachineId, registry::ConnectionState};

pub fn window_machine_offline_hint(state: &AppState, window_id: WindowId) -> Option<String> {
    let machine_id = state.window_machine_id(window_id)?;
    match machine_id {
        MachineId::Local => None,
        MachineId::Named(ref profile_id) => {
            let connected = state
                .machine_connections
                .get(profile_id)
                .is_some_and(|connection| *connection == ConnectionState::Connected);
            if connected {
                return None;
            }
            let label = state.host_label_for(&MachineId::Named(profile_id.clone()));
            Some(format!(
                "Not connected to {label}. Open Machines (m) and press c to connect."
            ))
        }
    }
}

pub fn enrich_load_error(state: &AppState, window_id: WindowId, error: &str) -> String {
    if error.contains("not connected")
        && let Some(hint) = window_machine_offline_hint(state, window_id)
    {
        return format!("{error}\n\n{hint}");
    }
    error.to_owned()
}
