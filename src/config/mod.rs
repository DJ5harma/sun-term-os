use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub mod launcher;
pub mod machines;
pub mod paths;
pub mod session;
pub mod theme;

pub use paths::{default_config_path, default_session_path};
pub use session::SessionConfig;
pub use theme::{ThemeConfig, parse_hex_color};

const MIN_REFRESH_SECS: u64 = 1;
const MAX_REFRESH_SECS: u64 = 300;
const MIN_WORKSPACES: usize = 1;
const MAX_WORKSPACES: usize = 9;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_refresh_interval_secs")]
    pub refresh_interval_secs: u64,
    #[serde(default = "default_workspace_count")]
    pub workspace_count: usize,
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub session: SessionConfig,
    #[serde(default)]
    pub machines: Vec<machines::MachineProfile>,
    #[serde(default)]
    pub launcher: launcher::LauncherConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            refresh_interval_secs: default_refresh_interval_secs(),
            workspace_count: default_workspace_count(),
            theme: ThemeConfig::default(),
            session: SessionConfig::default(),
            machines: Vec::new(),
            launcher: launcher::LauncherConfig::default(),
        }
    }
}

fn default_refresh_interval_secs() -> u64 {
    3
}

fn default_workspace_count() -> usize {
    3
}

impl Config {
    pub fn normalized(self) -> Self {
        Self {
            refresh_interval_secs: self
                .refresh_interval_secs
                .clamp(MIN_REFRESH_SECS, MAX_REFRESH_SECS),
            workspace_count: self.workspace_count.clamp(MIN_WORKSPACES, MAX_WORKSPACES),
            theme: self.theme,
            session: self.session,
            machines: self.machines,
            launcher: self.launcher,
        }
    }
}

pub fn load() -> Result<Config> {
    let path = default_config_path();
    if !path.exists() {
        return Ok(Config::default());
    }
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("read config {}", path.display()))?;
    let config = toml::from_str::<Config>(&text)
        .with_context(|| format!("parse config {}", path.display()))?;
    Ok(config.normalized())
}

pub fn save(config: &Config) -> Result<()> {
    let path = default_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create config dir {}", parent.display()))?;
    }
    let config = config.clone().normalized();
    let text = toml::to_string_pretty(&config).context("serialize config")?;
    std::fs::write(&path, text).with_context(|| format!("write config {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = Config::default();
        assert_eq!(config.refresh_interval_secs, 3);
        assert_eq!(config.workspace_count, 3);
    }

    #[test]
    fn normalizes_out_of_range_values() {
        let config = Config {
            refresh_interval_secs: 0,
            workspace_count: 20,
            theme: ThemeConfig::default(),
            session: SessionConfig::default(),
            machines: Vec::new(),
            launcher: launcher::LauncherConfig::default(),
        }
        .normalized();
        assert_eq!(config.refresh_interval_secs, MIN_REFRESH_SECS);
        assert_eq!(config.workspace_count, MAX_WORKSPACES);
    }

    #[test]
    fn parses_minimal_toml() {
        let parsed = toml::from_str::<Config>("refresh_interval_secs = 10\nworkspace_count = 5")
            .expect("parse");
        assert_eq!(parsed.refresh_interval_secs, 10);
        assert_eq!(parsed.workspace_count, 5);
    }
}
