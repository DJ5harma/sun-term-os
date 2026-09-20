use async_trait::async_trait;

use crate::machine::CapabilityError;
use crate::machine::applications::desktop::{desktop_search_paths, scan_desktop_dirs};
use crate::machine::applications::{ApplicationEntry, ApplicationProvider};

#[derive(Debug, Default)]
pub struct LocalApplicationProvider;

#[async_trait]
impl ApplicationProvider for LocalApplicationProvider {
    async fn discover(&self) -> Result<Vec<ApplicationEntry>, CapabilityError> {
        let paths = desktop_search_paths();
        tokio::task::spawn_blocking(move || scan_desktop_dirs(&paths))
            .await
            .map_err(|error| CapabilityError::Failed(error.to_string()))?
    }
}
