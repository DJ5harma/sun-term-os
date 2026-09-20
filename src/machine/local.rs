use async_trait::async_trait;
use sysinfo::{ProcessesToUpdate, System};

use super::{CapabilityError, ProcessInfo, ProcessProvider, SystemInfoProvider, SystemSnapshot};

#[derive(Debug, Default)]
pub struct LocalSystemInfoProvider;

#[async_trait]
impl SystemInfoProvider for LocalSystemInfoProvider {
    async fn snapshot(&self) -> Result<SystemSnapshot, CapabilityError> {
        let snapshot = tokio::task::spawn_blocking(|| {
            let mut system = System::new_all();
            system.refresh_all();

            SystemSnapshot {
                hostname: System::host_name().unwrap_or_else(|| "unknown host".to_owned()),
                os: System::long_os_version().unwrap_or_else(|| "unknown OS".to_owned()),
                kernel: System::kernel_version().unwrap_or_else(|| "unknown kernel".to_owned()),
                cpu_count: system.cpus().len(),
                memory_used: system.used_memory(),
                memory_total: system.total_memory(),
                uptime_seconds: System::uptime(),
            }
        })
        .await
        .map_err(|error| CapabilityError::Failed(error.to_string()))?;

        Ok(snapshot)
    }
}

#[derive(Debug, Default)]
pub struct LocalProcessProvider;

#[async_trait]
impl ProcessProvider for LocalProcessProvider {
    async fn processes(&self) -> Result<Vec<ProcessInfo>, CapabilityError> {
        let processes = tokio::task::spawn_blocking(|| {
            let mut system = System::new_all();
            system.refresh_processes(ProcessesToUpdate::All, true);

            let mut processes: Vec<_> = system
                .processes()
                .iter()
                .map(|(pid, process)| ProcessInfo {
                    pid: pid.as_u32(),
                    name: process.name().to_string_lossy().into_owned(),
                    cpu_percent: process.cpu_usage(),
                    memory_bytes: process.memory(),
                    state: process.status().to_string(),
                })
                .collect();
            processes.sort_by(|left, right| right.cpu_percent.total_cmp(&left.cpu_percent));
            processes.truncate(100);
            processes
        })
        .await
        .map_err(|error| CapabilityError::Failed(error.to_string()))?;

        Ok(processes)
    }
}
