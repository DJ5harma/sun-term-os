use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineProfile {
    pub id: String,
    pub label: String,
    pub host: String,
    #[serde(default = "default_ssh_port")]
    pub port: u16,
    pub user: String,
    #[serde(default)]
    pub identity_file: Option<PathBuf>,
}

fn default_ssh_port() -> u16 {
    22
}

impl MachineProfile {
    pub fn display_label(&self) -> &str {
        if self.label.is_empty() {
            &self.host
        } else {
            &self.label
        }
    }

    pub fn ssh_argv(&self) -> Vec<String> {
        let mut argv = vec![
            "ssh".to_owned(),
            "-t".to_owned(),
            "-p".to_owned(),
            self.port.to_string(),
        ];
        if let Some(path) = &self.identity_file {
            argv.push("-i".to_owned());
            argv.push(path.to_string_lossy().into_owned());
        }
        argv.push(format!("{}@{}", self.user, self.host));
        argv
    }
}
