use async_trait::async_trait;

use crate::machine::CapabilityError;
use crate::machine::services::{ServiceInfo, ServiceProvider};

use super::exec::ExecSession;

#[derive(Clone)]
pub struct RemoteServiceProvider {
    exec: ExecSession,
}

impl RemoteServiceProvider {
    pub fn new(exec: ExecSession) -> Self {
        Self { exec }
    }
}

#[async_trait]
impl ServiceProvider for RemoteServiceProvider {
    async fn list_services(&self) -> Result<Vec<ServiceInfo>, CapabilityError> {
        let output = self
            .exec
            .run("systemctl --user list-units --type=service --all --no-legend --plain --no-pager")
            .await
            .map_err(CapabilityError::Failed)?;
        Ok(crate::machine::local::services::parse_systemctl_list_public(&output))
    }

    async fn start(&self, name: &str) -> Result<(), CapabilityError> {
        self.run_systemctl(&["--user", "start", name]).await
    }

    async fn stop(&self, name: &str) -> Result<(), CapabilityError> {
        self.run_systemctl(&["--user", "stop", name]).await
    }

    async fn restart(&self, name: &str) -> Result<(), CapabilityError> {
        self.run_systemctl(&["--user", "restart", name]).await
    }
}

impl RemoteServiceProvider {
    async fn run_systemctl(&self, args: &[&str]) -> Result<(), CapabilityError> {
        let command = format!("systemctl {}", args.join(" "));
        let output = self.exec.run(&command).await;
        match output {
            Ok(_) => Ok(()),
            Err(error) => Err(CapabilityError::Failed(error)),
        }
    }
}
