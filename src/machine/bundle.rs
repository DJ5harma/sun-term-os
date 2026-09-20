use std::sync::Arc;

use crate::machine::{FilesystemProvider, ProcessProvider, SystemInfoProvider};

use super::local::{LocalFilesystemProvider, LocalProcessProvider, LocalSystemInfoProvider};

/// Local capability providers (filesystem, processes, system info).
#[derive(Clone)]
pub struct Machine {
    pub system: Arc<dyn SystemInfoProvider>,
    pub processes: Arc<dyn ProcessProvider>,
    pub filesystem: Arc<dyn FilesystemProvider>,
}

impl Machine {
    pub fn local() -> Self {
        Self {
            system: Arc::new(LocalSystemInfoProvider),
            processes: Arc::new(LocalProcessProvider),
            filesystem: Arc::new(LocalFilesystemProvider),
        }
    }
}
