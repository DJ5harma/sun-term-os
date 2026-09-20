use serde::{Deserialize, Serialize};

/// Identifies a machine whose capabilities the desktop can use.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum MachineId {
    #[default]
    Local,
    Named(String),
}

impl MachineId {
    pub fn is_local(&self) -> bool {
        matches!(self, MachineId::Local)
    }
}
