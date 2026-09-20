use async_trait::async_trait;
use sysinfo::{Disks, ProcessesToUpdate, Signal, System};

pub mod filesystem;

pub use filesystem::{LocalFilesystemProvider, default_start_path};

use super::{
    CapabilityError, DiskSnapshot, ProcessInfo, ProcessProvider, SystemInfoProvider, SystemSnapshot,
};

#[derive(Debug, Default)]
pub struct LocalSystemInfoProvider;

#[async_trait]
impl SystemInfoProvider for LocalSystemInfoProvider {
    async fn snapshot(&self) -> Result<SystemSnapshot, CapabilityError> {
        let snapshot = tokio::task::spawn_blocking(|| {
            let mut system = System::new_all();
            system.refresh_all();

            let disks = Disks::new_with_refreshed_list();
            let disks = disks
                .iter()
                .filter(|disk| !disk.is_removable())
                .map(|disk| DiskSnapshot {
                    name: disk.name().to_string_lossy().into_owned(),
                    mount_point: disk.mount_point().to_string_lossy().into_owned(),
                    total_bytes: disk.total_space(),
                    available_bytes: disk.available_space(),
                })
                .collect();

            SystemSnapshot {
                hostname: System::host_name().unwrap_or_else(|| "unknown host".to_owned()),
                os: System::long_os_version().unwrap_or_else(|| "unknown OS".to_owned()),
                kernel: System::kernel_version().unwrap_or_else(|| "unknown kernel".to_owned()),
                cpu_count: system.cpus().len(),
                memory_used: system.used_memory(),
                memory_total: system.total_memory(),
                uptime_seconds: System::uptime(),
                disks,
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

    async fn kill_process(&self, pid: u32) -> Result<(), CapabilityError> {
        let pid = pid;
        tokio::task::spawn_blocking(move || {
            let mut system = System::new();
            system.refresh_processes(
                ProcessesToUpdate::Some(&[sysinfo::Pid::from_u32(pid)]),
                true,
            );
            let process = system
                .process(sysinfo::Pid::from_u32(pid))
                .ok_or_else(|| CapabilityError::Failed(format!("process {pid} not found")))?;
            match process.kill_with(Signal::Term) {
                Some(true) => Ok(()),
                _ => Err(CapabilityError::Failed(format!("could not signal PID {pid}"))),
            }
        })
        .await
        .map_err(|error| CapabilityError::Failed(error.to_string()))?
    }
}
