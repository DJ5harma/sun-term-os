use crate::config::machines::MachineProfile;
use crate::machine::known_hosts::HostKeyError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MachinesDialog {
    None,
    AddProfile {
        input: String,
    },
    EditProfile {
        profile_id: String,
        input: String,
    },
    DeleteConfirm {
        profile_id: String,
        label: String,
    },
    HostKeyConfirm {
        profile_id: String,
        host: String,
        port: u16,
        fingerprint: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MachinesState {
    pub selected: usize,
    pub dialog: MachinesDialog,
    pub pending_host_key: Option<HostKeyError>,
}

impl MachinesState {
    pub fn new() -> Self {
        Self {
            selected: 0,
            dialog: MachinesDialog::None,
            pending_host_key: None,
        }
    }

    pub fn row_count(profiles: &[MachineProfile]) -> usize {
        profiles.len() + 1
    }
}
