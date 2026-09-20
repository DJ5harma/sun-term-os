use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Frame, layout::Rect};

use crate::{
    actions::{Action, ProcessAction, ShellAction},
    app::{
        effects::{Effect, ProcessEffect},
        process_manager::ProcessSortColumn,
        state::AppState,
    },
    domain::{ApplicationKind, Window, WindowId},
    ui::interaction::InteractionMap,
};

use super::{AppKeyResult, BuiltInApp, shell_keys::quick_launch_action};

pub const APP: BuiltInApp = BuiltInApp {
    kind: ApplicationKind::Processes,
    title: "Process Manager",
    short_title: "Proc",
    description: "View running processes",
    quick_launch_key: Some('p'),
    on_open,
    on_close,
    on_open_window: None,
    render,
    dispatch_key,
    desktop_scroll: Some(|delta| Action::Process(ProcessAction::ProcessPageScroll(delta))),
    palette_extras: None,
};

pub fn on_open(state: &mut AppState, window_id: WindowId) -> Vec<Effect> {
    let machine_id = state
        .window_machine_id(window_id)
        .unwrap_or(crate::machine::MachineId::Local);
    let listing = state.processes_for(&machine_id);
    state.init_process_manager(window_id, listing);
    Vec::new()
}

pub fn on_close(state: &mut AppState, window_id: WindowId) -> Vec<Effect> {
    state.remove_process_manager(window_id);
    Vec::new()
}

pub fn dispatch_key(key: KeyEvent, state: &AppState) -> AppKeyResult {
    let filter_active = state
        .focused_window()
        .filter(|window| window.application == ApplicationKind::Processes)
        .and_then(|window| state.process_manager(window.id))
        .is_some_and(|manager| manager.filter_active);

    let action = if filter_active {
        filter_key(key)
    } else {
        manager_key(key)
    };

    match action {
        Some(action) => AppKeyResult::Action(action),
        None => AppKeyResult::Consumed,
    }
}

fn manager_key(key: KeyEvent) -> Option<Action> {
    match key {
        KeyEvent {
            code: KeyCode::Up, ..
        } => Some(Action::Process(ProcessAction::MoveProcessSelection(-1))),
        KeyEvent {
            code: KeyCode::Down,
            ..
        } => Some(Action::Process(ProcessAction::MoveProcessSelection(1))),
        KeyEvent {
            code: KeyCode::PageUp,
            ..
        } => Some(Action::Process(ProcessAction::ProcessPageScroll(-1))),
        KeyEvent {
            code: KeyCode::PageDown,
            ..
        } => Some(Action::Process(ProcessAction::ProcessPageScroll(1))),
        KeyEvent {
            code: KeyCode::Char('/'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::Process(ProcessAction::ProcessFilterBegin)),
        KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: KeyModifiers::SHIFT,
            ..
        } => Some(Action::Process(ProcessAction::ProcessKillForceSelected)),
        KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::Process(ProcessAction::ProcessKillSelected)),
        KeyEvent {
            code: KeyCode::Char('1'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::Process(ProcessAction::ProcessSetSort(
            ProcessSortColumn::Cpu,
        ))),
        KeyEvent {
            code: KeyCode::Char('2'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::Process(ProcessAction::ProcessSetSort(
            ProcessSortColumn::Memory,
        ))),
        KeyEvent {
            code: KeyCode::Char('3'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::Process(ProcessAction::ProcessSetSort(
            ProcessSortColumn::Name,
        ))),
        KeyEvent {
            code: KeyCode::Char('4'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::Process(ProcessAction::ProcessSetSort(
            ProcessSortColumn::Pid,
        ))),
        KeyEvent {
            code: KeyCode::Char('r'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::Shell(ShellAction::Refresh)),
        other => quick_launch_action(other),
    }
}

fn filter_key(key: KeyEvent) -> Option<Action> {
    match key {
        KeyEvent {
            code: KeyCode::Esc, ..
        } => Some(Action::Process(ProcessAction::ProcessFilterEnd)),
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => Some(Action::Process(ProcessAction::ProcessFilterEnd)),
        KeyEvent {
            code: KeyCode::Backspace,
            ..
        } => Some(Action::Process(ProcessAction::ProcessFilterBackspace)),
        KeyEvent {
            code: KeyCode::Char(character),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::Process(ProcessAction::ProcessFilterPush(character))),
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
    if let Some(manager) = state.process_manager(window.id) {
        crate::ui::processes::render(frame, area, state, window.id, manager, interactions);
    } else {
        use ratatui::widgets::Paragraph;
        frame.render_widget(
            Paragraph::new("Process manager is starting…").style(crate::ui::theme::muted()),
            area,
        );
    }
}

pub async fn run_effect(
    executor: &mut crate::app::runtime::EffectExecutor,
    state: &mut AppState,
    effect: ProcessEffect,
) {
    use crate::app::effects::ProcessEffect;
    match effect {
        ProcessEffect::Kill(window_id, pid) => {
            let result = match executor.machine_for_window(state, window_id) {
                Some(machine) => machine.processes.kill_process(pid).await,
                None => Err(crate::machine::CapabilityError::Failed(
                    process_machine_message(state, window_id),
                )),
            };
            match result {
                Ok(()) => state.status = format!("Sent SIGTERM to PID {pid}"),
                Err(error) => state.status = format!("Kill failed: {error}"),
            }
        }
        ProcessEffect::KillForce(window_id, pid) => {
            let result = match executor.machine_for_window(state, window_id) {
                Some(machine) => machine.processes.kill_process_force(pid).await,
                None => Err(crate::machine::CapabilityError::Failed(
                    process_machine_message(state, window_id),
                )),
            };
            match result {
                Ok(()) => state.status = format!("Sent SIGKILL to PID {pid}"),
                Err(error) => state.status = format!("Force kill failed: {error}"),
            }
        }
    }
    executor.refresh_capabilities(state).await;
}

fn process_machine_message(state: &AppState, window_id: WindowId) -> String {
    let machine_id = state
        .window_machine_id(window_id)
        .unwrap_or_else(|| state.active_machine_id.clone());
    match machine_id {
        crate::machine::MachineId::Local => "local machine unavailable".to_owned(),
        crate::machine::MachineId::Named(name) => format!("not connected to {name}"),
    }
}
