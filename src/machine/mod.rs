use async_trait::async_trait;
use thiserror::Error;

pub mod local;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineKind {
    Local,
    #[allow(dead_code)]
    SshRemote,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MachineId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MachineDescriptor {
    pub id: MachineId,
    pub name: String,
    pub kind: MachineKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SystemSnapshot {
    pub hostname: String,
    pub os: String,
    pub kernel: String,
    pub cpu_count: usize,
    pub memory_used: u64,
    pub memory_total: u64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub state: String,
}

#[derive(Debug, Error)]
pub enum CapabilityError {
    #[error("capability unavailable: {0}")]
    #[allow(dead_code)]
    Unavailable(String),
    #[error("capability failed: {0}")]
    Failed(String),
}

#[async_trait]
pub trait SystemInfoProvider: Send + Sync {
    async fn snapshot(&self) -> Result<SystemSnapshot, CapabilityError>;
}

#[async_trait]
pub trait ProcessProvider: Send + Sync {
    async fn processes(&self) -> Result<Vec<ProcessInfo>, CapabilityError>;
}
