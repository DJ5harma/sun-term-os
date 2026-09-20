use std::sync::Arc;

use russh::Disconnect;
use russh::client;
use russh::keys::PublicKeyOrCertificate;
use russh::keys::{PrivateKey, PrivateKeyWithHashAlg};
use russh_sftp::client::SftpSession;
use tokio::sync::Mutex;

use crate::config::machines::MachineProfile;
use crate::machine::bundle::Machine;
use crate::machine::known_hosts::{HostKeyError, KnownHosts};
use crate::machine::remote::applications::RemoteApplicationProvider;
use crate::machine::remote::exec::ExecSession;
use crate::machine::remote::filesystem::SftpFilesystemProvider;
use crate::machine::remote::process::RemoteProcessProvider;
use crate::machine::remote::services::RemoteServiceProvider;
use crate::machine::remote::system::RemoteSystemInfoProvider;

pub struct RemoteSession {
    pub profile: MachineProfile,
    pub machine: Machine,
    handle: Arc<Mutex<client::Handle<SshHandler>>>,
}

#[derive(Debug)]
pub enum ConnectError {
    HostKey(HostKeyError),
    Auth(String),
    Transport(String),
}

pub(crate) struct SshHandler {
    known_hosts: Arc<Mutex<KnownHosts>>,
    host: String,
    port: u16,
    last_unknown: Arc<Mutex<Option<HostKeyError>>>,
}

impl client::Handler for SshHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &PublicKeyOrCertificate,
    ) -> Result<bool, Self::Error> {
        let known = self.known_hosts.lock().await;
        match server_public_key {
            PublicKeyOrCertificate::PublicKey { key, .. } => {
                match known.verify(&self.host, self.port, key) {
                    Ok(()) => Ok(true),
                    Err(HostKeyError::Mismatch) => {
                        *self.last_unknown.lock().await = Some(HostKeyError::Mismatch);
                        Ok(false)
                    }
                    Err(error @ HostKeyError::Unknown { .. }) => {
                        *self.last_unknown.lock().await = Some(error);
                        Ok(false)
                    }
                }
            }
            PublicKeyOrCertificate::Certificate(_) => {
                *self.last_unknown.lock().await = Some(HostKeyError::Mismatch);
                Ok(false)
            }
        }
    }
}

pub async fn connect(
    profile: MachineProfile,
    known_hosts: Arc<Mutex<KnownHosts>>,
) -> Result<RemoteSession, ConnectError> {
    let host = profile.host.clone();
    let port = profile.port;
    let last_unknown = Arc::new(Mutex::new(None));
    let config = Arc::new(client::Config::default());
    let handler = SshHandler {
        known_hosts: known_hosts.clone(),
        host: host.clone(),
        port,
        last_unknown: last_unknown.clone(),
    };
    let mut session = match client::connect(config, (host.as_str(), port), handler).await {
        Ok(session) => session,
        Err(error) => {
            if let Some(host_key) = take_host_key_error(&last_unknown).await {
                return Err(ConnectError::HostKey(host_key));
            }
            return Err(ConnectError::Transport(error.to_string()));
        }
    };

    let key = Arc::new(load_identity(&profile)?);
    let auth = match session
        .authenticate_publickey(&profile.user, PrivateKeyWithHashAlg::new(key, None))
        .await
    {
        Ok(auth) => auth,
        Err(error) => {
            if let Some(host_key) = take_host_key_error(&last_unknown).await {
                return Err(ConnectError::HostKey(host_key));
            }
            return Err(ConnectError::Auth(error.to_string()));
        }
    };
    if !auth.success() {
        if let Some(host_key) = take_host_key_error(&last_unknown).await {
            return Err(ConnectError::HostKey(host_key));
        }
        return Err(ConnectError::Auth(
            "public key authentication failed".to_owned(),
        ));
    }

    let handle = Arc::new(Mutex::new(session));
    let locked = handle.lock().await;
    let channel = locked
        .channel_open_session()
        .await
        .map_err(|error| ConnectError::Transport(error.to_string()))?;
    channel
        .request_subsystem(true, "sftp")
        .await
        .map_err(|error| ConnectError::Transport(error.to_string()))?;
    let sftp = SftpSession::new(channel.into_stream())
        .await
        .map_err(|error| ConnectError::Transport(error.to_string()))?;
    drop(locked);

    let sftp = Arc::new(Mutex::new(sftp));
    let exec = ExecSession::new(handle.clone());
    let machine = Machine {
        system: Arc::new(RemoteSystemInfoProvider::new(exec.clone())),
        processes: Arc::new(RemoteProcessProvider::new(exec.clone())),
        filesystem: Arc::new(SftpFilesystemProvider::new(sftp)),
        applications: Arc::new(RemoteApplicationProvider::new(exec.clone())),
        services: Arc::new(RemoteServiceProvider::new(exec)),
    };

    Ok(RemoteSession {
        profile,
        machine,
        handle,
    })
}

async fn take_host_key_error(
    last_unknown: &Arc<Mutex<Option<HostKeyError>>>,
) -> Option<HostKeyError> {
    last_unknown.lock().await.clone()
}

fn load_identity(profile: &MachineProfile) -> Result<PrivateKey, ConnectError> {
    if let Some(path) = &profile.identity_file {
        let text = std::fs::read_to_string(path)
            .map_err(|error| ConnectError::Auth(format!("read identity file: {error}")))?;
        return PrivateKey::from_openssh(&text)
            .map_err(|error| ConnectError::Auth(format!("parse identity: {error}")));
    }
    let home = std::env::var("HOME").map_err(|_| ConnectError::Auth("HOME not set".to_owned()))?;
    let default = format!("{home}/.ssh/id_ed25519");
    let text = std::fs::read_to_string(&default)
        .or_else(|_| std::fs::read_to_string(format!("{home}/.ssh/id_rsa")))
        .map_err(|error| ConnectError::Auth(format!("no default SSH key: {error}")))?;
    PrivateKey::from_openssh(&text)
        .map_err(|error| ConnectError::Auth(format!("parse identity: {error}")))
}

pub async fn disconnect(session: RemoteSession) {
    let _host = session.profile.host;
    let handle = session.handle.lock().await;
    let _ = handle
        .disconnect(Disconnect::ByApplication, "", "English")
        .await;
}

pub async fn trust_fingerprint(
    known_hosts: Arc<Mutex<KnownHosts>>,
    host: &str,
    port: u16,
    fingerprint_b64: &str,
) -> Result<(), String> {
    known_hosts
        .lock()
        .await
        .trust_fingerprint(host, port, fingerprint_b64)
        .map_err(|error| error.to_string())
}
