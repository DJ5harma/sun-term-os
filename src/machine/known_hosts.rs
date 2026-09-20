use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use russh::keys::{PublicKey, PublicKeyBase64, parse_public_key_base64};

/// Simple host-key store: `host:port` → base64-encoded public key.
#[derive(Debug, Default)]
pub struct KnownHosts {
    path: PathBuf,
    entries: HashMap<String, String>,
}

impl KnownHosts {
    pub fn load(path: PathBuf) -> Self {
        let entries = if path.exists() {
            parse_file(&path).unwrap_or_default()
        } else {
            HashMap::new()
        };
        Self { path, entries }
    }

    pub fn verify(&self, host: &str, port: u16, key: &PublicKey) -> Result<(), HostKeyError> {
        let id = host_key(host, port);
        let encoded = key.public_key_base64();
        match self.entries.get(&id) {
            Some(expected) if *expected == encoded => Ok(()),
            Some(_) => Err(HostKeyError::Mismatch),
            None => Err(HostKeyError::Unknown {
                host: host.to_owned(),
                port,
                fingerprint: encoded,
            }),
        }
    }

    pub fn trust(&mut self, host: &str, port: u16, key: &PublicKey) -> Result<()> {
        let id = host_key(host, port);
        self.entries.insert(id, key.public_key_base64());
        self.save()?;
        Ok(())
    }

    pub fn trust_fingerprint(
        &mut self,
        host: &str,
        port: u16,
        fingerprint_b64: &str,
    ) -> Result<()> {
        let key = parse_public_key_base64(fingerprint_b64).context("parse host key")?;
        self.trust(host, port, &key)
    }

    fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create known_hosts dir {}", parent.display()))?;
        }
        let mut lines: Vec<_> = self.entries.iter().collect();
        lines.sort_by_key(|(k, _)| k.as_str());
        let body = lines
            .iter()
            .map(|(id, key)| format!("{id} {key}"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&self.path, format!("{body}\n"))
            .with_context(|| format!("write {}", self.path.display()))?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostKeyError {
    Mismatch,
    Unknown {
        host: String,
        port: u16,
        fingerprint: String,
    },
}

fn host_key(host: &str, port: u16) -> String {
    if port == 22 {
        host.to_owned()
    } else {
        format!("[{host}]:{port}")
    }
}

fn parse_file(path: &Path) -> Result<HashMap<String, String>> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut map = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((id, key)) = line.split_once(' ') else {
            continue;
        };
        map.insert(id.to_owned(), key.to_owned());
    }
    Ok(map)
}

pub fn default_known_hosts_path() -> PathBuf {
    crate::config::paths::config_dir().join("known_hosts")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_key_formats() {
        assert_eq!(host_key("example.com", 22), "example.com");
        assert_eq!(host_key("example.com", 2222), "[example.com]:2222");
    }
}
