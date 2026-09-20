use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Frame, layout::Rect};

use crate::{
    actions::{Action, ServicesAction},
    app::{
        effects::{Effect, ServicesEffect},
        state::AppState,
    },
    domain::{ApplicationKind, Window, WindowId},
    ui::interaction::InteractionMap,
};

use super::{AppKeyResult, BuiltInApp};

pub const APP: BuiltInApp = BuiltInApp {
    kind: ApplicationKind::Services,
    title: "Service Manager",
    short_title: "Svc",
    description: "Manage user systemd services",
    quick_launch_key: None,
    on_open,
    on_close,
    on_open_window: None,
    render,
    dispatch_key,
    desktop_scroll: Some(|delta| Action::Services(ServicesAction::ServicesPageScroll(delta))),
    palette_extras: None,
};

pub fn on_open(state: &mut AppState, window_id: WindowId) -> Vec<Effect> {
    state.init_services_view(window_id);
    let machine_id = state
        .window_machine_id(window_id)
        .unwrap_or(crate::machine::MachineId::Local);
    vec![Effect::Services(ServicesEffect::List(machine_id))]
}

pub fn on_close(state: &mut AppState, window_id: WindowId) -> Vec<Effect> {
    state.remove_services_view(window_id);
    Vec::new()
}

pub fn dispatch_key(key: KeyEvent, state: &AppState) -> AppKeyResult {
    let filter_active = state
        .focused_window()
        .filter(|window| window.application == ApplicationKind::Services)
        .and_then(|window| state.services_view(window.id))
        .is_some_and(|view| view.filter_active);
    let action = if filter_active {
        filter_key(key)
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
        } => Some(Action::Services(ServicesAction::MoveSelection(-1))),
        KeyEvent {
            code: KeyCode::Down,
            ..
        } => Some(Action::Services(ServicesAction::MoveSelection(1))),
        KeyEvent {
            code: KeyCode::Char('/'),
            ..
        } => Some(Action::Services(ServicesAction::ServicesFilterBegin)),
        KeyEvent {
            code: KeyCode::Char('s'),
            ..
        } => Some(Action::Services(ServicesAction::ServiceStartSelected)),
        KeyEvent {
            code: KeyCode::Char('x'),
            ..
        } => Some(Action::Services(ServicesAction::ServiceStopSelected)),
        KeyEvent {
            code: KeyCode::Char('r'),
            ..
        } => Some(Action::Services(ServicesAction::ServiceRestartSelected)),
        _ => None,
    }
}

fn filter_key(key: KeyEvent) -> Option<Action> {
    match key {
        KeyEvent {
            code: KeyCode::Esc | KeyCode::Enter,
            ..
        } => Some(Action::Services(ServicesAction::ServicesFilterEnd)),
        KeyEvent {
            code: KeyCode::Backspace,
            ..
        } => Some(Action::Services(ServicesAction::ServicesFilterBackspace)),
        KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::Services(ServicesAction::ServicesFilterPush(ch))),
        _ => None,
    }
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    window: &Window,
    interactions: &mut InteractionMap,
) {
    if let Some(view) = state.services_view(window.id) {
        crate::ui::services::render(frame, area, state, window.id, view, interactions);
    }
}

pub async fn run_effect(
    executor: &mut crate::app::runtime::EffectExecutor,
    state: &mut AppState,
    effect: ServicesEffect,
) {
    use crate::actions::AsyncAction;
    match effect {
        ServicesEffect::List(machine_id) => {
            let result = match executor.registry.require_machine(&machine_id) {
                Ok(machine) => machine.services.list_services().await,
                Err(message) => Err(crate::machine::CapabilityError::Failed(message)),
            };
            crate::app::reducer::reduce(
                state,
                Action::Async(AsyncAction::ServicesReady(
                    machine_id,
                    result.map_err(|error| error.to_string()),
                )),
            );
        }
        ServicesEffect::Start(machine_id, name) => {
            run_service_action(executor, state, machine_id, &name, "start").await;
        }
        ServicesEffect::Stop(machine_id, name) => {
            run_service_action(executor, state, machine_id, &name, "stop").await;
        }
        ServicesEffect::Restart(machine_id, name) => {
            run_service_action(executor, state, machine_id, &name, "restart").await;
        }
    }
}

async fn run_service_action(
    executor: &mut crate::app::runtime::EffectExecutor,
    state: &mut AppState,
    machine_id: crate::machine::MachineId,
    name: &str,
    action: &str,
) {
    let machine = match executor.registry.require_machine(&machine_id) {
        Ok(machine) => machine,
        Err(message) => {
            state.status = message;
            return;
        }
    };
    let result = match action {
        "start" => machine.services.start(name).await,
        "stop" => machine.services.stop(name).await,
        _ => machine.services.restart(name).await,
    };
    match result {
        Ok(()) => state.status = format!("Service {action} {name}"),
        Err(error) => state.status = format!("Service {action} failed: {error}"),
    }
    let list = machine.services.list_services().await;
    crate::app::reducer::reduce(
        state,
        Action::Async(crate::actions::AsyncAction::ServicesReady(
            machine_id,
            list.map_err(|error| error.to_string()),
        )),
    );
}
