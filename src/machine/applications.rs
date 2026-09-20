pub mod desktop;

use async_trait::async_trait;

use super::CapabilityError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationEntry {
    pub id: String,
    pub name: String,
    pub detail: String,
    pub exec: String,
    pub launch_in_terminal: bool,
}

#[async_trait]
pub trait ApplicationProvider: Send + Sync {
    async fn discover(&self) -> Result<Vec<ApplicationEntry>, CapabilityError>;
}
