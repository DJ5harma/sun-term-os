use async_trait::async_trait;

use super::CapabilityError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceInfo {
    pub name: String,
    pub description: String,
    pub load_state: String,
    pub active_state: String,
    pub sub_state: String,
}

#[async_trait]
pub trait ServiceProvider: Send + Sync {
    async fn list_services(&self) -> Result<Vec<ServiceInfo>, CapabilityError>;

    async fn start(&self, name: &str) -> Result<(), CapabilityError>;

    async fn stop(&self, name: &str) -> Result<(), CapabilityError>;

    async fn restart(&self, name: &str) -> Result<(), CapabilityError>;
}
