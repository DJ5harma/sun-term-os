use async_trait::async_trait;

use crate::machine::{CapabilityError, DiskSnapshot, SystemInfoProvider, SystemSnapshot};

use super::exec::ExecSession;

#[derive(Clone)]
pub struct RemoteSystemInfoProvider {
    exec: ExecSession,
}

impl RemoteSystemInfoProvider {
    pub fn new(exec: ExecSession) -> Self {
        Self { exec }
    }
}

#[async_trait]
impl SystemInfoProvider for RemoteSystemInfoProvider {
    async fn snapshot(&self) -> Result<SystemSnapshot, CapabilityError> {
        let script = r"hostname; uname -srm; nproc; awk '/MemTotal/{print $2}' /proc/meminfo; awk '/MemAvailable/{print $2}' /proc/meminfo; awk '{print $1}' /proc/uptime; awk '{print $1,$2,$3}' /proc/loadavg; df -B1 --output=target,size,avail 2>/dev/null | tail -n +2";
        let output = self
            .exec
            .run(script)
            .await
            .map_err(CapabilityError::Failed)?;
        parse_snapshot(&output)
    }
}

fn parse_snapshot(output: &str) -> Result<SystemSnapshot, CapabilityError> {
    let lines: Vec<&str> = output.lines().map(str::trim).collect();
    if lines.len() < 7 {
        return Err(CapabilityError::Failed(
            "unexpected remote system output".into(),
        ));
    }
    let hostname = lines[0].to_owned();
    let uname_parts: Vec<_> = lines[1].split_whitespace().collect();
    let kernel = uname_parts.first().unwrap_or(&"unknown").to_string();
    let os = if uname_parts.len() > 1 {
        uname_parts[1..].join(" ")
    } else {
        "Linux".to_owned()
    };
    let cpu_count = lines[2].parse().unwrap_or(1);
    let memory_total_kb = lines[3].parse::<u64>().unwrap_or(0);
    let memory_avail_kb = lines[4].parse::<u64>().unwrap_or(0);
    let memory_total = memory_total_kb * 1024;
    let memory_used = memory_total.saturating_sub(memory_avail_kb * 1024);
    let uptime_seconds = lines[5].parse::<f64>().unwrap_or(0.0) as u64;
    let load_parts: Vec<_> = lines[6].split_whitespace().collect();
    let load_one = load_parts
        .first()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0);
    let load_five = load_parts
        .get(1)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0);
    let load_fifteen = load_parts
        .get(2)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0);

    let mut disks = Vec::new();
    for line in lines.iter().skip(7) {
        let parts: Vec<_> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        let mount_point = parts[0].to_owned();
        let total_bytes = parts[1].parse().unwrap_or(0);
        let available_bytes = parts[2].parse().unwrap_or(0);
        disks.push(DiskSnapshot {
            name: mount_point.clone(),
            mount_point,
            total_bytes,
            available_bytes,
        });
    }

    Ok(SystemSnapshot {
        hostname,
        os,
        kernel,
        cpu_count,
        memory_used,
        memory_total,
        uptime_seconds,
        load_one,
        load_five,
        load_fifteen,
        disks,
    })
}
