use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Frame, layout::Rect};
use std::process::Command;

use crate::{
    actions::{Action, AsyncAction, LauncherAction},
    app::{
        effects::{Effect, LauncherEffect, TerminalEffect},
        state::AppState,
    },
    domain::{ApplicationKind, Window, WindowId},
    ui::interaction::InteractionMap,
};

use super::{AppKeyResult, BuiltInApp, launcher_catalog::merge_launcher_config};

pub const APP: BuiltInApp = BuiltInApp {
    kind: ApplicationKind::Launcher,
    title: "Application Launcher",
    short_title: "Apps",
    description: "Browse and launch applications",
    quick_launch_key: Some('l'),
    on_open,
    on_close,
    on_open_window: None,
    render,
    dispatch_key,
    desktop_scroll: Some(|delta| Action::Launcher(LauncherAction::LauncherPageScroll(delta))),
    palette_extras: None,
};

pub fn on_open(state: &mut AppState, window_id: WindowId) -> Vec<Effect> {
    state.init_launcher_view(window_id);
    let machine_id = state
        .window_machine_id(window_id)
        .unwrap_or(crate::machine::MachineId::Local);
    vec![Effect::Launcher(LauncherEffect::Discover(machine_id))]
}

pub fn on_close(state: &mut AppState, window_id: WindowId) -> Vec<Effect> {
    state.remove_launcher_view(window_id);
    Vec::new()
}

pub fn dispatch_key(key: KeyEvent, state: &AppState) -> AppKeyResult {
    let filter_active = state
        .focused_window()
        .filter(|window| window.application == ApplicationKind::Launcher)
        .and_then(|window| state.launcher_view(window.id))
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
        } => Some(Action::Launcher(LauncherAction::MoveSelection(-1))),
        KeyEvent {
            code: KeyCode::Down,
            ..
        } => Some(Action::Launcher(LauncherAction::MoveSelection(1))),
        KeyEvent {
            code: KeyCode::Char('/'),
            ..
        } => Some(Action::Launcher(LauncherAction::LauncherFilterBegin)),
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => Some(Action::Launcher(LauncherAction::LaunchSelected)),
        KeyEvent {
            code: KeyCode::Char('F'),
            modifiers: KeyModifiers::SHIFT,
            ..
        } => Some(Action::Launcher(LauncherAction::ToggleFavorite)),
        _ => None,
    }
}

fn filter_key(key: KeyEvent) -> Option<Action> {
    match key {
        KeyEvent {
            code: KeyCode::Esc, ..
        }
        | KeyEvent {
            code: KeyCode::Enter,
            ..
        } => Some(Action::Launcher(LauncherAction::LauncherFilterEnd)),
        KeyEvent {
            code: KeyCode::Backspace,
            ..
        } => Some(Action::Launcher(LauncherAction::LauncherFilterBackspace)),
        KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::Launcher(LauncherAction::LauncherFilterPush(ch))),
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
    if let Some(view) = state.launcher_view(window.id) {
        crate::ui::app_launcher::render(frame, area, state, view, interactions);
    }
}

pub async fn run_effect(
    executor: &mut crate::app::runtime::EffectExecutor,
    state: &mut AppState,
    effect: LauncherEffect,
) {
    match effect {
        LauncherEffect::Discover(machine_id) => {
            let result = match executor.registry.require_machine(&machine_id) {
                Ok(machine) => machine.applications.discover().await,
                Err(message) => Err(crate::machine::CapabilityError::Failed(message)),
            };
            let listing =
                result.map(|entries| merge_launcher_config(&state.config.launcher, entries));
            crate::app::reducer::reduce(
                state,
                Action::Async(AsyncAction::LauncherReady(
                    machine_id,
                    listing.map_err(|error| error.to_string()),
                )),
            );
        }
        LauncherEffect::Launch(entry) => {
            if entry.launch_in_terminal {
                let window_id =
                    crate::app::reducer::open_application(state, ApplicationKind::Terminal);
                super::terminal::run_effect(
                    executor,
                    state,
                    TerminalEffect::StartWithCommand(window_id, entry.exec),
                )
                .await;
            } else {
                let exec = entry.exec;
                let name = entry.name;
                let result = tokio::task::spawn_blocking(move || launch_desktop_exec(&exec)).await;
                match result {
                    Ok(Ok(())) => state.status = format!("Launched {name}"),
                    Ok(Err(error)) => state.status = format!("Launch failed: {error}"),
                    Err(error) => state.status = format!("Launch failed: {error}"),
                }
            }
        }
    }
}

fn launch_desktop_exec(exec: &str) -> Result<(), String> {
    let status = Command::new("sh").args(["-c", exec]).status();
    match status {
        Ok(exit) if exit.success() => Ok(()),
        Ok(exit) => Err(format!("exit code {}", exit)),
        Err(error) => Err(error.to_string()),
    }
}
