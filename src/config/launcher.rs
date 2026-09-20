use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LauncherExtraCommand {
    pub name: String,
    pub command: String,
    pub in_terminal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LauncherConfig {
    #[serde(default)]
    pub favorites: Vec<String>,
    #[serde(default)]
    pub extra_commands: Vec<LauncherExtraCommand>,
}
