use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Frame, layout::Rect};

use crate::{
    actions::{Action, MachinesAction},
    app::{
        effects::{Effect, MachinesEffect},
        machines::MachinesDialog,
        runtime::EffectExecutor,
        state::AppState,
    },
    config,
    domain::{ApplicationKind, Window, WindowId},
    machine::{MachineId, known_hosts::HostKeyError, remote::ConnectError},
    ui::interaction::InteractionMap,
};

use super::{AppKeyResult, BuiltInApp};

pub const APP: BuiltInApp = BuiltInApp {
    kind: ApplicationKind::Machines,
    title: "Machines",
    short_title: "Mach",
    description: "Connect to remote hosts over SSH",
    quick_launch_key: Some('m'),
    on_open,
    on_close,
    on_open_window: None,
    render,
    dispatch_key,
    desktop_scroll: None,
    palette_extras: None,
};

pub fn on_open(state: &mut AppState, window_id: WindowId) -> Vec<Effect> {
    state.init_machines_view(window_id);
    Vec::new()
}

pub fn on_close(state: &mut AppState, window_id: WindowId) -> Vec<Effect> {
    state.remove_machines_view(window_id);
    Vec::new()
}

pub fn dispatch_key(key: KeyEvent, state: &AppState) -> AppKeyResult {
    let dialog_active = state
        .focused_window()
        .filter(|window| window.application == ApplicationKind::Machines)
        .and_then(|window| state.machines_view(window.id))
        .is_some_and(|view| view.dialog != MachinesDialog::None);

    let action = if dialog_active {
        dialog_key(key)
    } else {
        main_key(key)
    };
    match action {
        Some(action) => AppKeyResult::Action(action),
        None => AppKeyResult::Consumed,
    }
}

fn main_key(key: KeyEvent) -> Option<Action> {
    match key {
        KeyEvent {
            code: KeyCode::Up, ..
        } => Some(Action::Machines(MachinesAction::MoveSelection(-1))),
        KeyEvent {
            code: KeyCode::Down,
            ..
        } => Some(Action::Machines(MachinesAction::MoveSelection(1))),
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => Some(Action::Machines(MachinesAction::SetActiveMachine)),
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::Machines(MachinesAction::ConnectSelected)),
        KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::Machines(MachinesAction::DisconnectSelected)),
        KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::Machines(MachinesAction::BeginAddProfile)),
        KeyEvent {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::Machines(MachinesAction::BeginEditProfile)),
        KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::Machines(MachinesAction::DeleteSelected)),
        KeyEvent {
            code: KeyCode::Char('y'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::Machines(MachinesAction::AcceptHostKey)),
        KeyEvent {
            code: KeyCode::Char('n'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::Machines(MachinesAction::RejectHostKey)),
        _ => None,
    }
}

fn dialog_key(key: KeyEvent) -> Option<Action> {
    match key {
        KeyEvent {
            code: KeyCode::Esc, ..
        } => Some(Action::Machines(MachinesAction::CancelDialog)),
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => Some(Action::Machines(MachinesAction::DialogCommit)),
        KeyEvent {
            code: KeyCode::Backspace,
            ..
        } => Some(Action::Machines(MachinesAction::DialogBackspace)),
        KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::Machines(MachinesAction::DialogPush(ch))),
        _ => None,
    }
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    window: &Window,
    _interactions: &mut InteractionMap,
) {
    if let Some(view) = state.machines_view(window.id) {
        crate::ui::machines::render(frame, area, state, view);
    }
}

pub async fn run_effect(
    executor: &mut EffectExecutor,
    state: &mut AppState,
    effect: MachinesEffect,
) {
    match effect {
        MachinesEffect::PersistConfig => {
            if let Err(error) = config::save(&state.config) {
                state.status = format!("Could not save machines: {error}");
            } else {
                state.status = "Machine profiles saved".to_owned();
            }
        }
        MachinesEffect::Connect(profile_id) => {
            if let Some(profile) =
                crate::machine::MachineRegistry::profile_by_id(&state.config.machines, &profile_id)
            {
                let result = executor.registry.connect_remote(profile).await;
                apply_connect_result(executor, state, profile_id, result).await;
            }
        }
        MachinesEffect::Disconnect(profile_id) => {
            let was_active = state.active_machine_id == MachineId::Named(profile_id.clone());
            executor.registry.disconnect_remote(&profile_id).await;
            sync_connections(executor, state);
            if was_active {
                state.active_machine_id = MachineId::Local;
            }
            state.status = "Disconnected".to_owned();
        }
        MachinesEffect::TrustHostKey {
            profile_id,
            host,
            port,
            fingerprint,
        } => {
            if let Err(error) = executor
                .registry
                .trust_host_key(&host, port, &fingerprint)
                .await
            {
                state.status = format!("Could not trust host key: {error}");
                return;
            }
            for view in state.machines_views.values_mut() {
                view.dialog = MachinesDialog::None;
            }
            if let Some(profile) =
                crate::machine::MachineRegistry::profile_by_id(&state.config.machines, &profile_id)
            {
                let result = executor.registry.connect_remote(profile).await;
                apply_connect_result(executor, state, profile_id, result).await;
            }
        }
    }
}

fn sync_connections(executor: &EffectExecutor, state: &mut AppState) {
    executor
        .registry
        .sync_connections_to(&state.config.machines, &mut state.machine_connections);
}

async fn apply_connect_result(
    executor: &mut EffectExecutor,
    state: &mut AppState,
    profile_id: String,
    result: Result<(), ConnectError>,
) {
    match result {
        Ok(()) => {
            state.active_machine_id = MachineId::Named(profile_id);
            state.status = format!("Connected to {}", state.host_label());
            sync_connections(executor, state);
            executor
                .refresh_machine(state, state.active_machine_id.clone())
                .await;
        }
        Err(ConnectError::HostKey(HostKeyError::Unknown {
            host,
            port,
            fingerprint,
        })) => {
            for view in state.machines_views.values_mut() {
                view.dialog = MachinesDialog::HostKeyConfirm {
                    profile_id: profile_id.clone(),
                    host: host.clone(),
                    port,
                    fingerprint: fingerprint.clone(),
                };
            }
            state.status = "Unknown host key · y accept · n reject".to_owned();
            sync_connections(executor, state);
        }
        Err(ConnectError::HostKey(HostKeyError::Mismatch)) => {
            state.status = "Host key mismatch — update known_hosts".to_owned();
            sync_connections(executor, state);
        }
        Err(ConnectError::Auth(message)) => {
            state.status = format!("SSH auth failed: {message}");
            sync_connections(executor, state);
        }
        Err(ConnectError::Transport(message)) => {
            state.status = format!("SSH connect failed: {message}");
            sync_connections(executor, state);
        }
    }
}
