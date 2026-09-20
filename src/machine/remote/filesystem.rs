use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use russh_sftp::client::SftpSession;
use tokio::sync::Mutex;

use crate::machine::{
    CapabilityError, DirectoryListing, FileEntry, FileEntryKind, FilesystemProvider, RemoveOutcome,
};

#[derive(Clone)]
pub struct SftpFilesystemProvider {
    sftp: Arc<Mutex<SftpSession>>,
}

impl SftpFilesystemProvider {
    pub fn new(sftp: Arc<Mutex<SftpSession>>) -> Self {
        Self { sftp }
    }
}

#[async_trait]
impl FilesystemProvider for SftpFilesystemProvider {
    async fn list_directory(
        &self,
        path: &Path,
        show_hidden: bool,
    ) -> Result<DirectoryListing, CapabilityError> {
        let path_str = path_to_remote(path);
        let sftp = self.sftp.lock().await;
        let read_dir = sftp
            .read_dir(&path_str)
            .await
            .map_err(|error| CapabilityError::Failed(error.to_string()))?;
        let mut entries = Vec::new();
        for entry in read_dir {
            let name = entry.file_name();
            if !show_hidden && name.starts_with('.') {
                continue;
            }
            let meta = entry.metadata();
            let kind = if meta.is_dir() {
                FileEntryKind::Directory
            } else if meta.is_symlink() {
                FileEntryKind::Symlink
            } else if meta.is_regular() {
                FileEntryKind::File
            } else {
                FileEntryKind::Other
            };
            entries.push(FileEntry {
                name,
                kind,
                size_bytes: Some(meta.len()),
                modified_secs: meta.mtime.map(|time| time as u64),
            });
        }
        entries.sort_by_key(|a| a.name.to_lowercase());
        Ok(DirectoryListing {
            path: path.to_path_buf(),
            entries,
        })
    }

    async fn remove_path(&self, path: &Path) -> Result<RemoveOutcome, CapabilityError> {
        let path_str = path_to_remote(path);
        let sftp = self.sftp.lock().await;
        if let Ok(meta) = sftp.metadata(&path_str).await {
            if meta.is_dir() {
                sftp.remove_dir(&path_str)
                    .await
                    .map_err(|error| CapabilityError::Failed(error.to_string()))?;
            } else {
                sftp.remove_file(&path_str)
                    .await
                    .map_err(|error| CapabilityError::Failed(error.to_string()))?;
            }
            return Ok(RemoveOutcome::DeletedPermanently);
        }
        Err(CapabilityError::Failed("path not found".into()))
    }

    async fn rename_path(&self, from: &Path, to: &Path) -> Result<(), CapabilityError> {
        let sftp = self.sftp.lock().await;
        sftp.rename(path_to_remote(from), path_to_remote(to))
            .await
            .map_err(|error| CapabilityError::Failed(error.to_string()))
    }

    async fn create_file(&self, path: &Path) -> Result<(), CapabilityError> {
        let path_str = path_to_remote(path);
        let sftp = self.sftp.lock().await;
        use russh_sftp::protocol::OpenFlags;
        let _file = sftp
            .open_with_flags(
                &path_str,
                OpenFlags::CREATE | OpenFlags::WRITE | OpenFlags::TRUNCATE,
            )
            .await
            .map_err(|error| CapabilityError::Failed(error.to_string()))?;
        Ok(())
    }

    async fn create_directory(&self, path: &Path) -> Result<(), CapabilityError> {
        let sftp = self.sftp.lock().await;
        sftp.create_dir(path_to_remote(path))
            .await
            .map_err(|error| CapabilityError::Failed(error.to_string()))
    }

    async fn read_text_file(&self, path: &Path, max_bytes: u64) -> Result<String, CapabilityError> {
        let path_str = path_to_remote(path);
        let sftp = self.sftp.lock().await;
        let meta = sftp
            .metadata(&path_str)
            .await
            .map_err(|error| CapabilityError::Failed(error.to_string()))?;
        if meta.len() > max_bytes {
            return Err(CapabilityError::Failed(format!(
                "file exceeds {max_bytes} byte limit"
            )));
        }
        let mut file = sftp
            .open(&path_str)
            .await
            .map_err(|error| CapabilityError::Failed(error.to_string()))?;
        let mut bytes = Vec::new();
        use tokio::io::AsyncReadExt;
        file.read_to_end(&mut bytes)
            .await
            .map_err(|error| CapabilityError::Failed(error.to_string()))?;
        if bytes.contains(&0) {
            return Err(CapabilityError::Failed(
                "binary file cannot be viewed".into(),
            ));
        }
        String::from_utf8(bytes).map_err(|_| CapabilityError::Failed("not valid UTF-8".into()))
    }
}

fn path_to_remote(path: &Path) -> String {
    let text = path.to_string_lossy();
    if text.is_empty() {
        ".".to_owned()
    } else {
        text.into_owned()
    }
}
