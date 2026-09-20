use crate::{
    actions::SettingsAction,
    app::{
        AppState,
        settings::SettingsRow,
    },
    domain::ApplicationKind,
    ui::theme,
};

use super::Effect;

pub(super) fn reduce(state: &mut AppState, action: SettingsAction) -> Vec<Effect> {
    match action {
        SettingsAction::MoveSelection(delta) => {
            if let Some(window_id) = focused_settings_window(state)
                && let Some(view) = state.settings_view_mut(window_id)
            {
                view.move_selection(delta);
            }
        }
        SettingsAction::AdjustRefresh(delta) => {
            if focused_settings_window(state).is_some() {
                let next = state.config.refresh_interval_secs as i64 + delta as i64;
                state.config.refresh_interval_secs = next.clamp(1, 300) as u64;
            }
        }
        SettingsAction::ActivateRow => {
            if let Some(window_id) = focused_settings_window(state) {
                let row = state
                    .settings_view(window_id)
                    .map(|view| view.selected_row())
                    .unwrap_or(SettingsRow::RefreshInterval);
                match row {
                    SettingsRow::RestoreSession => {
                        state.config.session.restore_on_start =
                            !state.config.session.restore_on_start;
                    }
                    SettingsRow::SaveSession => {
                        state.config.session.save_on_exit = !state.config.session.save_on_exit;
                    }
                    SettingsRow::ThemePreset => {
                        let next_index = state
                            .settings_view(window_id)
                            .map(|view| {
                                (view.theme_preset_index + 1)
                                    % crate::app::theme_presets::len()
                            });
                        if let Some(next_index) = next_index {
                            if let Some(view) = state.settings_view_mut(window_id) {
                                view.theme_preset_index = next_index;
                            }
                            crate::app::theme_presets::apply(
                                next_index,
                                &mut state.config.theme,
                            );
                            theme::reload(&state.config.theme);
                        }
                    }
                    SettingsRow::Save => return vec![Effect::PersistConfig],
                    SettingsRow::RefreshInterval => {}
                }
            }
        }
        SettingsAction::Save => return vec![Effect::PersistConfig],
    }
    Vec::new()
}

fn focused_settings_window(state: &AppState) -> Option<u64> {
    state
        .focused_window()
        .filter(|window| window.application == ApplicationKind::Settings)
        .map(|window| window.id)
}
