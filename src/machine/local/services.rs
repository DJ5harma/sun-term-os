use async_trait::async_trait;

use crate::machine::CapabilityError;
use crate::machine::services::{ServiceInfo, ServiceProvider};

#[derive(Debug, Default)]
pub struct LocalServiceProvider;

#[async_trait]
impl ServiceProvider for LocalServiceProvider {
    async fn list_services(&self) -> Result<Vec<ServiceInfo>, CapabilityError> {
        tokio::task::spawn_blocking(list_user_services)
            .await
            .map_err(|error| CapabilityError::Failed(error.to_string()))?
    }

    async fn start(&self, name: &str) -> Result<(), CapabilityError> {
        run_systemctl(&["--user", "start", name]).await
    }

    async fn stop(&self, name: &str) -> Result<(), CapabilityError> {
        run_systemctl(&["--user", "stop", name]).await
    }

    async fn restart(&self, name: &str) -> Result<(), CapabilityError> {
        run_systemctl(&["--user", "restart", name]).await
    }
}

fn list_user_services() -> Result<Vec<ServiceInfo>, CapabilityError> {
    if !command_exists("systemctl") {
        return Err(CapabilityError::Unavailable(
            "systemctl is not available on this machine".into(),
        ));
    }
    let output = std::process::Command::new("systemctl")
        .args([
            "--user",
            "list-units",
            "--type=service",
            "--all",
            "--no-legend",
            "--plain",
            "--no-pager",
        ])
        .output()
        .map_err(|error| CapabilityError::Failed(error.to_string()))?;
    if !output.status.success() {
        return Err(CapabilityError::Failed(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(parse_systemctl_list_public(&text))
}

pub fn parse_systemctl_list_public(text: &str) -> Vec<ServiceInfo> {
    let mut services = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<_> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }
        let name = parts[0].to_owned();
        let load_state = parts[1].to_owned();
        let active_state = parts[2].to_owned();
        let sub_state = parts[3].to_owned();
        services.push(ServiceInfo {
            name: name.clone(),
            description: name,
            load_state,
            active_state,
            sub_state,
        });
    }
    services
}

async fn run_systemctl(args: &[&str]) -> Result<(), CapabilityError> {
    if !command_exists("systemctl") {
        return Err(CapabilityError::Unavailable(
            "systemctl is not available on this machine".into(),
        ));
    }
    let args = args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>();
    tokio::task::spawn_blocking(move || {
        let output = std::process::Command::new("systemctl")
            .args(&args)
            .output()
            .map_err(|error| CapabilityError::Failed(error.to_string()))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(CapabilityError::Failed(
                String::from_utf8_lossy(&output.stderr).into_owned(),
            ))
        }
    })
    .await
    .map_err(|error| CapabilityError::Failed(error.to_string()))?
}

fn command_exists(command: &str) -> bool {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {command}"))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
