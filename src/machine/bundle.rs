use std::sync::Arc;

use crate::machine::{
    ApplicationProvider, FilesystemProvider, ProcessProvider, ServiceProvider, SystemInfoProvider,
};

use super::local::{
    LocalApplicationProvider, LocalFilesystemProvider, LocalProcessProvider, LocalServiceProvider,
    LocalSystemInfoProvider,
};

/// Local capability providers (filesystem, processes, system info).
#[derive(Clone)]
pub struct Machine {
    pub system: Arc<dyn SystemInfoProvider>,
    pub processes: Arc<dyn ProcessProvider>,
    pub filesystem: Arc<dyn FilesystemProvider>,
    pub applications: Arc<dyn ApplicationProvider>,
    pub services: Arc<dyn ServiceProvider>,
}

impl Machine {
    pub fn local() -> Self {
        Self {
            system: Arc::new(LocalSystemInfoProvider),
            processes: Arc::new(LocalProcessProvider),
            filesystem: Arc::new(LocalFilesystemProvider),
            applications: Arc::new(LocalApplicationProvider),
            services: Arc::new(LocalServiceProvider),
        }
    }
}
