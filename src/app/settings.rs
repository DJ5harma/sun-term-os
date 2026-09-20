use crate::app::theme_presets;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsRow {
    RefreshInterval,
    RestoreSession,
    SaveSession,
    ThemePreset,
    Save,
}

impl SettingsRow {
    pub const ALL: [SettingsRow; 5] = [
        SettingsRow::RefreshInterval,
        SettingsRow::RestoreSession,
        SettingsRow::SaveSession,
        SettingsRow::ThemePreset,
        SettingsRow::Save,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::RefreshInterval => "Refresh interval (seconds)",
            Self::RestoreSession => "Restore session on start",
            Self::SaveSession => "Save session on exit",
            Self::ThemePreset => "Color theme",
            Self::Save => "Save settings to disk",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SettingsState {
    pub selected: usize,
    /// Which built-in palette row is active in Settings (not stored in config).
    pub theme_preset_index: usize,
}

impl SettingsState {
    pub fn selected_row(&self) -> SettingsRow {
        SettingsRow::ALL[self.selected.min(SettingsRow::ALL.len() - 1)]
    }

    pub fn move_selection(&mut self, delta: i32) {
        let len = SettingsRow::ALL.len();
        let next = self.selected as i32 + delta;
        self.selected = next.rem_euclid(len as i32) as usize;
    }
}

pub fn display_value(
    config: &crate::config::Config,
    view: &SettingsState,
    row: SettingsRow,
) -> String {
    match row {
        SettingsRow::RefreshInterval => config.refresh_interval_secs.to_string(),
        SettingsRow::RestoreSession => on_off(config.session.restore_on_start),
        SettingsRow::SaveSession => on_off(config.session.save_on_exit),
        SettingsRow::ThemePreset => {
            format!(
                "{} · Enter for next",
                theme_presets::label(view.theme_preset_index)
            )
        }
        SettingsRow::Save => "Enter or s · ~/.config/sun-term-os/config.toml".to_owned(),
    }
}

fn on_off(value: bool) -> String {
    if value {
        "on".to_owned()
    } else {
        "off".to_owned()
    }
}
