use crate::{
    actions::ServicesAction,
    app::{AppState, Loadable},
    domain::ApplicationKind,
    machine::services::ServiceInfo,
};

use super::Effect;

pub(super) fn reduce(state: &mut AppState, action: ServicesAction) -> Vec<Effect> {
    let window_id = focused_services_window(state);
    match action {
        ServicesAction::MoveSelection(offset) => {
            if let Some(id) = window_id {
                let count = filtered_services(state, id).len();
                if let Some(view) = state.services_view_mut(id) {
                    let next = view.selected_index as i32 + offset;
                    view.selected_index = next.clamp(0, count as i32 - 1).max(0) as usize;
                }
            }
        }
        ServicesAction::ServicesPageScroll(pages) => {
            if let Some(id) = window_id {
                let count = filtered_services(state, id).len();
                if let Some(view) = state.services_view_mut(id) {
                    let delta = pages * view.visible_rows.max(1) as i32;
                    let next = view.selected_index as i32 + delta;
                    view.selected_index = next.clamp(0, count as i32 - 1).max(0) as usize;
                }
            }
        }
        ServicesAction::ServicesFilterBegin => {
            if let Some(id) = window_id
                && let Some(view) = state.services_view_mut(id)
            {
                view.filter_active = true;
            }
        }
        ServicesAction::ServicesFilterPush(ch) => {
            if let Some(id) = window_id
                && let Some(view) = state.services_view_mut(id)
            {
                view.filter.push(ch);
            }
        }
        ServicesAction::ServicesFilterBackspace => {
            if let Some(id) = window_id
                && let Some(view) = state.services_view_mut(id)
            {
                view.filter.pop();
            }
        }
        ServicesAction::ServicesFilterEnd => {
            if let Some(id) = window_id
                && let Some(view) = state.services_view_mut(id)
            {
                view.filter_active = false;
                view.selected_index = 0;
            }
        }
        ServicesAction::ServiceStartSelected
        | ServicesAction::ServiceStopSelected
        | ServicesAction::ServiceRestartSelected => {
            if let Some(id) = window_id {
                let machine_id = state
                    .window_machine_id(id)
                    .unwrap_or(crate::machine::MachineId::Local);
                let services = filtered_services(state, id);
                let index = state
                    .services_view(id)
                    .map(|view| view.selected_index)
                    .unwrap_or(0);
                if let Some(service) = services.get(index) {
                    let effect = match action {
                        ServicesAction::ServiceStartSelected => {
                            crate::app::effects::ServicesEffect::Start(
                                machine_id,
                                service.name.clone(),
                            )
                        }
                        ServicesAction::ServiceStopSelected => {
                            crate::app::effects::ServicesEffect::Stop(
                                machine_id,
                                service.name.clone(),
                            )
                        }
                        ServicesAction::ServiceRestartSelected => {
                            crate::app::effects::ServicesEffect::Restart(
                                machine_id,
                                service.name.clone(),
                            )
                        }
                        _ => unreachable!(),
                    };
                    return vec![crate::app::effects::Effect::Services(effect)];
                }
            }
        }
    }
    Vec::new()
}

fn focused_services_window(state: &AppState) -> Option<u64> {
    state
        .focused_window()
        .filter(|window| window.application == ApplicationKind::Services)
        .map(|window| window.id)
}

fn filtered_services(state: &AppState, window_id: u64) -> Vec<ServiceInfo> {
    let view = state.services_view(window_id);
    let listing = view
        .map(|view| view.listing.clone())
        .unwrap_or(Loadable::Loading);
    let filter = view
        .map(|view| view.filter.to_lowercase())
        .unwrap_or_default();
    match listing {
        Loadable::Ready(services) => services
            .into_iter()
            .filter(|service| {
                filter.is_empty()
                    || service.name.to_lowercase().contains(&filter)
                    || service.active_state.to_lowercase().contains(&filter)
            })
            .collect(),
        _ => Vec::new(),
    }
}
