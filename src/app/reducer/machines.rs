use crate::{
    actions::MachinesAction,
    app::{
        AppState,
        effects::{Effect, MachinesEffect},
        machines::MachinesDialog,
    },
    config::machines::MachineProfile,
    domain::ApplicationKind,
    machine::MachineId,
};

use super::Effect as EffectVec;

pub(super) fn reduce(state: &mut AppState, action: MachinesAction) -> Vec<EffectVec> {
    let window_id = focused_machines_window(state);
    match action {
        MachinesAction::MoveSelection(offset) => {
            if let Some(id) = window_id {
                let count = crate::app::machines::MachinesState::row_count(&state.config.machines);
                if let Some(view) = state.machines_view_mut(id) {
                    let next = view.selected as i32 + offset;
                    view.selected = next.clamp(0, count as i32 - 1) as usize;
                }
            }
        }
        MachinesAction::SetActiveMachine => {
            if let Some(id) = window_id {
                let row = state
                    .machines_view(id)
                    .map(|view| view.selected)
                    .unwrap_or(0);
                state.active_machine_id = if row == 0 {
                    MachineId::Local
                } else if let Some(profile) = state.config.machines.get(row - 1) {
                    MachineId::Named(profile.id.clone())
                } else {
                    MachineId::Local
                };
                state.status = format!("Active machine: {}", state.host_label());
            }
        }
        MachinesAction::ConnectSelected => {
            if let Some(profile) = selected_profile(state, window_id) {
                return vec![Effect::Machines(MachinesEffect::Connect(
                    profile.id.clone(),
                ))];
            }
        }
        MachinesAction::DisconnectSelected => {
            if let Some(profile) = selected_profile(state, window_id) {
                return vec![Effect::Machines(MachinesEffect::Disconnect(
                    profile.id.clone(),
                ))];
            }
        }
        MachinesAction::BeginAddProfile => {
            if let Some(id) = window_id
                && let Some(view) = state.machines_view_mut(id)
            {
                view.dialog = MachinesDialog::AddProfile {
                    input: String::new(),
                };
            }
        }
        MachinesAction::BeginEditProfile => {
            if let (Some(id), Some(profile)) = (window_id, selected_profile(state, window_id))
                && let Some(view) = state.machines_view_mut(id)
            {
                view.dialog = MachinesDialog::EditProfile {
                    profile_id: profile.id.clone(),
                    input: profile.host.clone(),
                };
            }
        }
        MachinesAction::DeleteSelected => {
            if let (Some(id), Some(profile)) = (window_id, selected_profile(state, window_id))
                && let Some(view) = state.machines_view_mut(id)
            {
                view.dialog = MachinesDialog::DeleteConfirm {
                    profile_id: profile.id.clone(),
                    label: profile.display_label().to_owned(),
                };
            }
        }
        MachinesAction::AcceptHostKey => {
            if let Some(id) = window_id {
                let dialog = state.machines_view(id).map(|view| view.dialog.clone());
                if let Some(MachinesDialog::HostKeyConfirm {
                    profile_id,
                    host,
                    port,
                    fingerprint,
                }) = dialog
                {
                    return vec![Effect::Machines(MachinesEffect::TrustHostKey {
                        profile_id,
                        host,
                        port,
                        fingerprint,
                    })];
                }
            }
        }
        MachinesAction::RejectHostKey => {
            if let Some(id) = window_id {
                if let Some(view) = state.machines_view_mut(id) {
                    view.dialog = MachinesDialog::None;
                }
                state.status = "Host key rejected".to_owned();
            }
        }
        MachinesAction::DialogPush(ch) => dialog_push(state, window_id, ch),
        MachinesAction::DialogBackspace => dialog_backspace(state, window_id),
        MachinesAction::DialogCommit => return dialog_commit(state, window_id),
        MachinesAction::CancelDialog => {
            if let Some(id) = window_id
                && let Some(view) = state.machines_view_mut(id)
            {
                view.dialog = MachinesDialog::None;
            }
        }
    }
    Vec::new()
}

fn focused_machines_window(state: &AppState) -> Option<u64> {
    state
        .focused_window()
        .filter(|window| window.application == ApplicationKind::Machines)
        .map(|window| window.id)
}

fn selected_profile(state: &AppState, window_id: Option<u64>) -> Option<MachineProfile> {
    let id = window_id?;
    let row = state
        .machines_view(id)
        .map(|view| view.selected)
        .unwrap_or(0);
    if row == 0 {
        None
    } else {
        state.config.machines.get(row - 1).cloned()
    }
}

fn dialog_push(state: &mut AppState, window_id: Option<u64>, ch: char) {
    if let Some(id) = window_id
        && let Some(view) = state.machines_view_mut(id)
    {
        match &mut view.dialog {
            MachinesDialog::AddProfile { input, .. }
            | MachinesDialog::EditProfile { input, .. } => input.push(ch),
            _ => {}
        }
    }
}

fn dialog_backspace(state: &mut AppState, window_id: Option<u64>) {
    if let Some(id) = window_id
        && let Some(view) = state.machines_view_mut(id)
    {
        match &mut view.dialog {
            MachinesDialog::AddProfile { input, .. }
            | MachinesDialog::EditProfile { input, .. } => {
                input.pop();
            }
            _ => {}
        }
    }
}

fn dialog_commit(state: &mut AppState, window_id: Option<u64>) -> Vec<EffectVec> {
    let Some(id) = window_id else {
        return Vec::new();
    };
    let dialog = state.machines_view(id).map(|view| view.dialog.clone());
    match dialog {
        Some(MachinesDialog::DeleteConfirm { profile_id, .. }) => {
            state
                .config
                .machines
                .retain(|profile| profile.id != profile_id);
            if let Some(view) = state.machines_view_mut(id) {
                view.dialog = MachinesDialog::None;
            }
            state.status = "Profile removed".to_owned();
            vec![Effect::Machines(MachinesEffect::PersistConfig)]
        }
        Some(MachinesDialog::AddProfile { input, .. }) if !input.is_empty() => {
            let profile = MachineProfile {
                id: input.replace('.', "-"),
                label: input.clone(),
                host: input.clone(),
                port: 22,
                user: default_user(),
                identity_file: None,
            };
            state.config.machines.push(profile);
            if let Some(view) = state.machines_view_mut(id) {
                view.dialog = MachinesDialog::None;
            }
            state.status = "Profile added".to_owned();
            vec![Effect::Machines(MachinesEffect::PersistConfig)]
        }
        Some(MachinesDialog::EditProfile {
            profile_id, input, ..
        }) => {
            if let Some(profile) = state
                .config
                .machines
                .iter_mut()
                .find(|profile| profile.id == profile_id)
            {
                profile.host = input.clone();
                if profile.label.is_empty() {
                    profile.label = input.clone();
                }
            }
            if let Some(view) = state.machines_view_mut(id) {
                view.dialog = MachinesDialog::None;
            }
            vec![Effect::Machines(MachinesEffect::PersistConfig)]
        }
        _ => Vec::new(),
    }
}

fn default_user() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "root".to_owned())
}
