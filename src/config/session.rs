use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::domain::ApplicationKind;
use crate::machine::MachineId;

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
    pub active_machine_id: Option<String>,
    #[serde(default)]
    pub connected_machine_ids: Vec<String>,
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
    pub machine_id: Option<String>,
    #[serde(default)]
    pub file_manager_path: Option<PathBuf>,
    #[serde(default)]
    pub text_viewer_path: Option<PathBuf>,
}

pub fn encode_machine_id(machine_id: &MachineId) -> String {
    match machine_id {
        MachineId::Local => "local".to_owned(),
        MachineId::Named(name) => name.clone(),
    }
}

pub fn decode_machine_id(value: Option<&str>) -> MachineId {
    match value {
        None | Some("local") => MachineId::Local,
        Some(name) => MachineId::Named(name.to_owned()),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine_id_round_trip() {
        assert_eq!(encode_machine_id(&MachineId::Local), "local");
        assert_eq!(decode_machine_id(Some("local")), MachineId::Local);
        assert_eq!(decode_machine_id(None), MachineId::Local);
        let named = MachineId::Named("prod".to_owned());
        assert_eq!(encode_machine_id(&named), "prod");
        assert_eq!(decode_machine_id(Some("prod")), named);
    }
}
