use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            refresh_interval_secs: default_refresh_interval_secs(),
            workspace_count: default_workspace_count(),
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
        }
    }
}

pub fn default_config_path() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(dir).join("tde/config.toml");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".config/tde/config.toml");
    }
    PathBuf::from("config.toml")
}

pub fn load(path: Option<&Path>) -> Result<Config> {
    let path = path.map(PathBuf::from).unwrap_or_else(default_config_path);
    if !path.exists() {
        return Ok(Config::default());
    }
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("read config {}", path.display()))?;
    let config = toml::from_str::<Config>(&text)
        .with_context(|| format!("parse config {}", path.display()))?;
    Ok(config.normalized())
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
