use crate::{
    actions::ProcessAction,
    app::{
        AppState, ApplicationKind, Loadable,
        process_manager::{matching_indices, selected_process},
    },
};

use super::Effect;

pub(super) fn reduce(state: &mut AppState, action: ProcessAction) -> Vec<Effect> {
    match action {
        ProcessAction::MoveProcessSelection(offset) => {
            if let Some(window_id) = focused_process_window(state) {
                let count = process_match_count(state, window_id);
                if let Some(manager) = state.process_manager_mut(window_id)
                    && count > 0
                {
                    let next = manager.selected_index as i32 + offset;
                    manager.selected_index = next.clamp(0, count as i32 - 1) as usize;
                    manager.clamp_selection(count);
                }
            }
        }
        ProcessAction::ProcessPageScroll(pages) => {
            if let Some(window_id) = focused_process_window(state) {
                let count = process_match_count(state, window_id);
                if let Some(manager) = state.process_manager_mut(window_id) {
                    let delta = pages * manager.visible_rows.max(1) as i32;
                    let next = manager.selected_index as i32 + delta;
                    manager.selected_index = next.clamp(0, count.saturating_sub(1) as i32) as usize;
                    manager.clamp_selection(count);
                }
            }
        }
        ProcessAction::ProcessFilterBegin => {
            if let Some(window_id) = focused_process_window(state)
                && let Some(manager) = state.process_manager_mut(window_id)
            {
                manager.filter_active = true;
                manager.filter.clear();
                manager.selected_index = 0;
                manager.scroll_offset = 0;
                state.status = "Process filter · type to search · Esc when done".to_owned();
            }
        }
        ProcessAction::ProcessFilterPush(character) => {
            if let Some(window_id) = focused_process_window(state) {
                if let Some(manager) = state.process_manager_mut(window_id) {
                    manager.filter.push(character);
                }
                let count = process_match_count(state, window_id);
                if let Some(manager) = state.process_manager_mut(window_id) {
                    manager.clamp_selection(count);
                }
            }
        }
        ProcessAction::ProcessFilterBackspace => {
            if let Some(window_id) = focused_process_window(state) {
                if let Some(manager) = state.process_manager_mut(window_id) {
                    manager.filter.pop();
                }
                let count = process_match_count(state, window_id);
                if let Some(manager) = state.process_manager_mut(window_id) {
                    manager.clamp_selection(count);
                }
            }
        }
        ProcessAction::ProcessFilterEnd => {
            if let Some(window_id) = focused_process_window(state)
                && let Some(manager) = state.process_manager_mut(window_id)
            {
                manager.filter_active = false;
                state.status = "Process filter applied".to_owned();
            }
        }
        ProcessAction::ProcessKillSelected => {
            let kill = focused_process_window(state).and_then(|window_id| {
                state.process_manager(window_id).and_then(|manager| {
                    let Loadable::Ready(processes) = &manager.listing else {
                        return None;
                    };
                    selected_process(
                        processes,
                        &manager.filter,
                        manager.sort,
                        manager.selected_index,
                    )
                    .map(|process| (process.pid, process.name.clone()))
                })
            });
            if let Some((pid, name)) = kill {
                state.status = format!("Sending SIGTERM to {name} ({pid})");
                return vec![Effect::Process(crate::app::effects::ProcessEffect::Kill(
                    pid,
                ))];
            }
            state.status = "No process selected".to_owned();
        }
        ProcessAction::ProcessKillForceSelected => {
            let kill = focused_process_window(state).and_then(|window_id| {
                state.process_manager(window_id).and_then(|manager| {
                    let Loadable::Ready(processes) = &manager.listing else {
                        return None;
                    };
                    selected_process(
                        processes,
                        &manager.filter,
                        manager.sort,
                        manager.selected_index,
                    )
                    .map(|process| (process.pid, process.name.clone()))
                })
            });
            if let Some((pid, name)) = kill {
                state.status = format!("Sending SIGKILL to {name} ({pid})");
                return vec![Effect::Process(
                    crate::app::effects::ProcessEffect::KillForce(pid),
                )];
            }
            state.status = "No process selected".to_owned();
        }
        ProcessAction::ProcessSetSort(sort) => {
            if let Some(window_id) = focused_process_window(state) {
                if let Some(manager) = state.process_manager_mut(window_id) {
                    manager.sort = sort;
                }
                let count = process_match_count(state, window_id);
                if let Some(manager) = state.process_manager_mut(window_id) {
                    manager.clamp_selection(count);
                }
                state.status = format!("Sort by {}", sort.label());
            }
        }
        ProcessAction::SelectProcessRow(window_id, index) => {
            if focused_process_window(state) == Some(window_id) {
                let count = process_match_count(state, window_id);
                if let Some(manager) = state.process_manager_mut(window_id)
                    && index < count
                {
                    manager.selected_index = index;
                    manager.clamp_selection(count);
                }
            }
        }
    }
    Vec::new()
}

fn focused_process_window(state: &AppState) -> Option<u64> {
    state
        .focused_window()
        .filter(|window| window.application == ApplicationKind::Processes)
        .map(|window| window.id)
}

fn process_match_count(state: &AppState, window_id: u64) -> usize {
    let Some(manager) = state.process_manager(window_id) else {
        return 0;
    };
    match &manager.listing {
        Loadable::Ready(processes) => {
            matching_indices(processes, &manager.filter, manager.sort).len()
        }
        _ => 0,
    }
}
