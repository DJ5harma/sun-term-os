use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::domain::ApplicationKind;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionConfig {
    #[serde(default = "default_restore_session")]
    pub restore_on_start: bool,
    #[serde(default = "default_save_session")]
    pub save_on_exit: bool,
}

fn default_restore_session() -> bool {
    true
}

fn default_save_session() -> bool {
    true
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            restore_on_start: default_restore_session(),
            save_on_exit: default_save_session(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SessionFile {
    #[serde(default)]
    pub active_workspace: usize,
    #[serde(default)]
    pub workspaces: Vec<SessionWorkspace>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SessionWorkspace {
    #[serde(default)]
    pub windows: Vec<SessionWindow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionWindow {
    pub application: ApplicationKind,
    #[serde(default)]
    pub file_manager_path: Option<PathBuf>,
}

pub fn session_path() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(dir).join("tde/session.toml");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".config/tde/session.toml");
    }
    PathBuf::from("session.toml")
}

pub fn load_session(path: &Path) -> Result<SessionFile> {
    if !path.exists() {
        return Ok(SessionFile::default());
    }
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read session {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("parse session {}", path.display()))
}

pub fn save_session(path: &Path, session: &SessionFile) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create session dir {}", parent.display()))?;
    }
    let text = toml::to_string_pretty(session).context("serialize session")?;
    std::fs::write(path, text).with_context(|| format!("write session {}", path.display()))
}
