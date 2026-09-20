use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::config::machines::MachineProfile;
use crate::machine::bundle::Machine;
use crate::machine::id::MachineId;
use crate::machine::known_hosts::{KnownHosts, default_known_hosts_path};
use crate::machine::remote::{ConnectError, RemoteSession, connect, disconnect};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Failed(String),
}

pub struct MachineRegistry {
    pub local: Machine,
    known_hosts: Arc<Mutex<KnownHosts>>,
    remotes: HashMap<String, RemoteSession>,
    pub connection_state: HashMap<String, ConnectionState>,
}

impl MachineRegistry {
    pub fn new() -> Self {
        Self {
            local: Machine::local(),
            known_hosts: Arc::new(Mutex::new(KnownHosts::load(default_known_hosts_path()))),
            remotes: HashMap::new(),
            connection_state: HashMap::new(),
        }
    }

    pub fn machine(&self, id: &MachineId) -> Option<&Machine> {
        match id {
            MachineId::Local => Some(&self.local),
            MachineId::Named(name) => self.remotes.get(name).map(|session| &session.machine),
        }
    }

    pub fn require_machine(&self, id: &MachineId) -> Result<&Machine, String> {
        match self.machine(id) {
            Some(machine) => Ok(machine),
            None => Err(match id {
                MachineId::Local => "local machine unavailable".to_owned(),
                MachineId::Named(name) => format!("not connected to {name}"),
            }),
        }
    }

    pub fn sync_connections_to(
        &self,
        profiles: &[MachineProfile],
        out: &mut HashMap<String, ConnectionState>,
    ) {
        for profile in profiles {
            let id = MachineId::Named(profile.id.clone());
            out.insert(profile.id.clone(), self.connection_state(&id));
        }
    }

    pub fn connection_state(&self, id: &MachineId) -> ConnectionState {
        match id {
            MachineId::Local => ConnectionState::Connected,
            MachineId::Named(name) => self
                .connection_state
                .get(name)
                .cloned()
                .unwrap_or(ConnectionState::Disconnected),
        }
    }

    pub fn set_connecting(&mut self, profile_id: &str) {
        self.connection_state
            .insert(profile_id.to_owned(), ConnectionState::Connecting);
    }

    pub fn set_failed(&mut self, profile_id: &str, message: String) {
        self.connection_state
            .insert(profile_id.to_owned(), ConnectionState::Failed(message));
    }

    pub async fn connect_remote(&mut self, profile: MachineProfile) -> Result<(), ConnectError> {
        let id = profile.id.clone();
        self.set_connecting(&id);
        if let Some(session) = self.remotes.remove(&id) {
            disconnect(session).await;
        }
        match connect(profile, self.known_hosts.clone()).await {
            Ok(session) => {
                self.remotes.insert(id.clone(), session);
                self.connection_state.insert(id, ConnectionState::Connected);
                Ok(())
            }
            Err(error) => {
                let message = match &error {
                    ConnectError::HostKey(_) => "unknown or mismatched host key".to_owned(),
                    ConnectError::Auth(msg) => msg.clone(),
                    ConnectError::Transport(msg) => msg.clone(),
                };
                self.set_failed(&id, message);
                Err(error)
            }
        }
    }

    pub async fn disconnect_remote(&mut self, profile_id: &str) {
        if let Some(session) = self.remotes.remove(profile_id) {
            disconnect(session).await;
        }
        self.connection_state
            .insert(profile_id.to_owned(), ConnectionState::Disconnected);
    }

    pub async fn trust_host_key(
        &self,
        host: &str,
        port: u16,
        fingerprint: &str,
    ) -> Result<(), String> {
        crate::machine::remote::trust_fingerprint(self.known_hosts.clone(), host, port, fingerprint)
            .await
    }

    pub fn profile_by_id(profiles: &[MachineProfile], id: &str) -> Option<MachineProfile> {
        profiles.iter().find(|profile| profile.id == id).cloned()
    }
}
