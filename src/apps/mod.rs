//! Built-in applications: lifecycle, render, input, effects, and palette contributions.

mod file_manager;
pub(crate) mod launcher;
mod launcher_catalog;
mod machines;
mod processes;
mod services;
mod settings;
pub(crate) mod shell_keys;
mod system_info;
mod terminal;
pub(crate) mod text_viewer;

use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};

use crate::{
    actions::{Action, ShellAction},
    app::{effects::Effect, palette::PaletteEntry, state::AppState},
    domain::{ApplicationKind, Window, WindowId},
    ui::interaction::InteractionMap,
};

#[derive(Debug, Clone, PartialEq)]
pub enum AppKeyResult {
    Action(Action),
    Terminal(Vec<u8>),
    Consumed,
}

type OpenFn = fn(&mut AppState, WindowId) -> Vec<Effect>;
type CloseFn = fn(&mut AppState, WindowId) -> Vec<Effect>;
type OpenWindowFn = fn(&mut AppState, WindowId);
type RenderFn = fn(&mut Frame, Rect, &AppState, &Window, &mut InteractionMap);
type KeyFn = fn(KeyEvent, &AppState) -> AppKeyResult;
type ScrollFn = fn(i32) -> Action;
type PaletteExtrasFn = fn(&AppState) -> Vec<PaletteEntry>;

#[derive(Debug, Clone, Copy)]
pub struct BuiltInApp {
    pub kind: ApplicationKind,
    pub title: &'static str,
    pub short_title: &'static str,
    pub description: &'static str,
    pub quick_launch_key: Option<char>,
    pub on_open: OpenFn,
    pub on_close: CloseFn,
    pub on_open_window: Option<OpenWindowFn>,
    pub render: RenderFn,
    pub dispatch_key: KeyFn,
    pub desktop_scroll: Option<ScrollFn>,
    pub palette_extras: Option<PaletteExtrasFn>,
}

const BUILT_INS: [BuiltInApp; 9] = [
    terminal::APP,
    file_manager::APP,
    processes::APP,
    system_info::APP,
    settings::APP,
    machines::APP,
    launcher::APP,
    text_viewer::APP,
    services::APP,
];

pub fn all() -> &'static [BuiltInApp] {
    &BUILT_INS
}

pub fn by_kind(kind: ApplicationKind) -> Option<&'static BuiltInApp> {
    BUILT_INS.iter().find(|app| app.kind == kind)
}

pub fn by_quick_launch_key(character: char) -> Option<&'static BuiltInApp> {
    BUILT_INS
        .iter()
        .find(|app| app.quick_launch_key == Some(character))
}

pub fn title(kind: ApplicationKind) -> &'static str {
    by_kind(kind).map(|app| app.title).unwrap_or("Application")
}

pub fn short_title(kind: ApplicationKind) -> &'static str {
    by_kind(kind).map(|app| app.short_title).unwrap_or("App")
}

pub fn open_action(kind: ApplicationKind) -> Action {
    Action::Shell(ShellAction::OpenApplication(kind))
}

pub fn palette_entry(app: &BuiltInApp) -> PaletteEntry {
    PaletteEntry {
        title: app.title.to_owned(),
        detail: app.description.to_owned(),
        action: open_action(app.kind),
    }
}

pub fn palette_extras(state: &AppState) -> Vec<PaletteEntry> {
    BUILT_INS
        .iter()
        .filter_map(|app| app.palette_extras)
        .flat_map(|extras| extras(state))
        .collect()
}

pub fn dispatch_key(kind: ApplicationKind, key: KeyEvent, state: &AppState) -> AppKeyResult {
    by_kind(kind)
        .map(|app| (app.dispatch_key)(key, state))
        .unwrap_or(AppKeyResult::Consumed)
}

pub fn desktop_scroll_action(kind: ApplicationKind, delta: i32) -> Option<Action> {
    by_kind(kind).and_then(|app| app.desktop_scroll.map(|scroll| scroll(delta)))
}

pub fn on_open_window(state: &mut AppState, kind: ApplicationKind, window_id: WindowId) {
    if let Some(app) = by_kind(kind)
        && let Some(hook) = app.on_open_window
    {
        hook(state, window_id);
    }
}

pub fn open_effects(
    kind: ApplicationKind,
    state: &mut AppState,
    window_id: WindowId,
) -> Vec<Effect> {
    by_kind(kind)
        .map(|app| (app.on_open)(state, window_id))
        .unwrap_or_default()
}

pub fn on_close(kind: ApplicationKind, state: &mut AppState, window_id: WindowId) -> Vec<Effect> {
    by_kind(kind)
        .map(|app| (app.on_close)(state, window_id))
        .unwrap_or_default()
}

pub fn render(
    kind: ApplicationKind,
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    window: &Window,
    interactions: &mut InteractionMap,
) {
    if let Some(app) = by_kind(kind) {
        (app.render)(frame, area, state, window, interactions);
    }
}

pub async fn run_effect(
    executor: &mut crate::app::runtime::EffectExecutor,
    state: &mut AppState,
    effect: Effect,
) {
    use crate::app::effects::Effect;
    match effect {
        Effect::PersistConfig => {
            if let Err(error) = crate::config::save(&state.config) {
                state.status = format!("Could not save settings: {error}");
            } else {
                crate::ui::theme::reload(&state.config.theme);
                state.status = "Settings saved".to_owned();
            }
        }
        Effect::RefreshCapabilities => executor.refresh_capabilities(state).await,
        Effect::Terminal(effect) => terminal::run_effect(executor, state, effect).await,
        Effect::FileManager(effect) => file_manager::run_effect(executor, state, effect).await,
        Effect::Process(effect) => processes::run_effect(executor, state, effect).await,
        Effect::Machines(effect) => machines::run_effect(executor, state, effect).await,
        Effect::Launcher(effect) => launcher::run_effect(executor, state, effect).await,
        Effect::TextViewer(effect) => text_viewer::run_effect(executor, state, effect).await,
        Effect::Services(effect) => services::run_effect(executor, state, effect).await,
    }
}
