use async_trait::async_trait;

use crate::machine::{CapabilityError, ProcessInfo, ProcessProvider};

use super::exec::ExecSession;

#[derive(Clone)]
pub struct RemoteProcessProvider {
    exec: ExecSession,
}

impl RemoteProcessProvider {
    pub fn new(exec: ExecSession) -> Self {
        Self { exec }
    }
}

#[async_trait]
impl ProcessProvider for RemoteProcessProvider {
    async fn processes(&self) -> Result<Vec<ProcessInfo>, CapabilityError> {
        let output = self
            .exec
            .run("ps -eo pid=,pcpu=,pmem=,state=,comm= --no-headers")
            .await
            .map_err(CapabilityError::Failed)?;
        Ok(parse_ps(&output))
    }

    async fn kill_process(&self, pid: u32) -> Result<(), CapabilityError> {
        self.exec
            .run(&format!("kill {pid}"))
            .await
            .map_err(CapabilityError::Failed)?;
        Ok(())
    }

    async fn kill_process_force(&self, pid: u32) -> Result<(), CapabilityError> {
        self.exec
            .run(&format!("kill -9 {pid}"))
            .await
            .map_err(CapabilityError::Failed)?;
        Ok(())
    }
}

fn parse_ps(output: &str) -> Vec<ProcessInfo> {
    let mut processes = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts: Vec<_> = line.split_whitespace().collect();
        if parts.len() < 5 {
            continue;
        }
        let pid = parts.remove(0).parse().unwrap_or(0);
        let cpu_percent = parts.remove(0).parse().unwrap_or(0.0);
        let mem_percent = parts.remove(0).parse::<f32>().unwrap_or(0.0);
        let state = parts.remove(0).to_owned();
        let name = parts.join(" ");
        processes.push(ProcessInfo {
            pid,
            name,
            cpu_percent,
            memory_bytes: (mem_percent * 10_000_000.0) as u64,
            state,
        });
    }
    processes
}
