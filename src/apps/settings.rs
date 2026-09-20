use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Frame, layout::Rect};

use crate::{
    actions::{Action, SettingsAction},
    app::{effects::Effect, state::AppState},
    domain::{ApplicationKind, Window, WindowId},
    ui::interaction::InteractionMap,
};

use super::{AppKeyResult, BuiltInApp, shell_keys};

pub const APP: BuiltInApp = BuiltInApp {
    kind: ApplicationKind::Settings,
    title: "Settings",
    short_title: "Set",
    description: "Appearance and behavior",
    quick_launch_key: None,
    on_open,
    on_close,
    on_open_window: None,
    render,
    dispatch_key,
    desktop_scroll: None,
    palette_extras: None,
};

pub fn on_open(state: &mut AppState, window_id: WindowId) -> Vec<Effect> {
    state.init_settings_view(window_id);
    Vec::new()
}

pub fn on_close(state: &mut AppState, window_id: WindowId) -> Vec<Effect> {
    state.remove_settings_view(window_id);
    Vec::new()
}

pub fn dispatch_key(key: KeyEvent, _state: &AppState) -> AppKeyResult {
    let action = match key {
        KeyEvent {
            code: KeyCode::Up, ..
        } => Some(Action::Settings(SettingsAction::MoveSelection(-1))),
        KeyEvent {
            code: KeyCode::Down,
            ..
        } => Some(Action::Settings(SettingsAction::MoveSelection(1))),
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => Some(Action::Settings(SettingsAction::ActivateRow)),
        KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::Settings(SettingsAction::Save)),
        KeyEvent {
            code: KeyCode::Char('+') | KeyCode::Char('='),
            ..
        } => Some(Action::Settings(SettingsAction::AdjustRefresh(1))),
        KeyEvent {
            code: KeyCode::Char('-'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Action::Settings(SettingsAction::AdjustRefresh(-1))),
        other => shell_keys::quick_launch_action(other),
    };
    match action {
        Some(action) => AppKeyResult::Action(action),
        None => AppKeyResult::Consumed,
    }
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    window: &Window,
    _interactions: &mut InteractionMap,
) {
    if let Some(view) = state.settings_view(window.id) {
        crate::ui::settings::render(frame, area, state, view);
    } else {
        use ratatui::widgets::Paragraph;
        frame.render_widget(
            Paragraph::new("Settings…").style(crate::ui::theme::muted()),
            area,
        );
    }
}
