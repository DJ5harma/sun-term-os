use std::path::{Path, PathBuf};

use async_trait::async_trait;

use super::super::{
    CapabilityError, DirectoryListing, FileEntry, FileEntryKind, FilesystemProvider,
};

#[derive(Debug, Default)]
pub struct LocalFilesystemProvider;

#[async_trait]
impl FilesystemProvider for LocalFilesystemProvider {
    async fn list_directory(
        &self,
        path: &Path,
        show_hidden: bool,
    ) -> Result<DirectoryListing, CapabilityError> {
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || list_directory_blocking(&path, show_hidden))
            .await
            .map_err(|error| CapabilityError::Failed(error.to_string()))?
    }
}

fn list_directory_blocking(
    path: &Path,
    show_hidden: bool,
) -> Result<DirectoryListing, CapabilityError> {
    if !path.exists() {
        return Err(CapabilityError::Failed(format!(
            "Path does not exist: {}",
            path.display()
        )));
    }
    if !path.is_dir() {
        return Err(CapabilityError::Failed(format!(
            "Not a directory: {}",
            path.display()
        )));
    }

    let read_dir = std::fs::read_dir(path).map_err(map_io_error)?;
    let mut entries = Vec::new();

    for entry in read_dir {
        let entry = entry.map_err(map_io_error)?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !show_hidden && name.starts_with('.') {
            continue;
        }
        let metadata = entry.metadata().map_err(map_io_error)?;
        let kind = if metadata.is_dir() {
            FileEntryKind::Directory
        } else if metadata.is_symlink() {
            FileEntryKind::Symlink
        } else if metadata.is_file() {
            FileEntryKind::File
        } else {
            FileEntryKind::Other
        };
        let size_bytes = if metadata.is_file() {
            Some(metadata.len())
        } else {
            None
        };
        let modified_secs = metadata.modified().ok().and_then(|time| {
            time.duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| d.as_secs())
        });
        entries.push(FileEntry {
            name,
            kind,
            size_bytes,
            modified_secs,
        });
    }

    sort_entries(&mut entries);

    Ok(DirectoryListing {
        path: path.to_path_buf(),
        entries,
    })
}

fn sort_entries(entries: &mut [FileEntry]) {
    entries.sort_by(|left, right| {
        let left_dir = left.kind == FileEntryKind::Directory;
        let right_dir = right.kind == FileEntryKind::Directory;
        right_dir
            .cmp(&left_dir)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });
}

fn map_io_error(error: std::io::Error) -> CapabilityError {
    let message = match error.kind() {
        std::io::ErrorKind::PermissionDenied => format!("Permission denied: {error}"),
        std::io::ErrorKind::NotFound => format!("Not found: {error}"),
        _ => error.to_string(),
    };
    CapabilityError::Failed(message)
}

pub fn default_start_path() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        let path = PathBuf::from(home);
        if path.is_dir() {
            return path;
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn touch(path: &Path) {
        fs::write(path, b"").expect("write file");
    }

    #[test]
    fn lists_and_sorts_entries_with_hidden_filter() {
        let root = tempfile::tempdir().expect("tempdir");
        let root_path = root.path();
        touch(&root_path.join("zebra.txt"));
        fs::create_dir(root_path.join("Beta")).expect("mkdir");
        touch(&root_path.join(".hidden"));
        fs::create_dir(root_path.join("alpha")).expect("mkdir");

        let visible = list_directory_blocking(root_path, false).expect("list");
        assert_eq!(visible.entries.len(), 3);
        assert!(visible.entries.iter().all(|e| !e.name.starts_with('.')));
        assert_eq!(visible.entries[0].name, "alpha");
        assert_eq!(visible.entries[1].name, "Beta");
        assert_eq!(visible.entries[2].name, "zebra.txt");

        let all = list_directory_blocking(root_path, true).expect("list hidden");
        assert_eq!(all.entries.len(), 4);
        assert!(all.entries.iter().any(|e| e.name == ".hidden"));
    }

    #[test]
    fn missing_path_returns_failed_capability() {
        let err = list_directory_blocking(Path::new("/no/such/path/for/tde-test"), false)
            .expect_err("missing");
        assert!(matches!(err, CapabilityError::Failed(_)));
    }

    #[test]
    fn file_metadata_includes_modified_when_available() {
        let root = tempfile::tempdir().expect("tempdir");
        let file = root.path().join("dated.txt");
        touch(&file);
        let listing = list_directory_blocking(root.path(), false).expect("list");
        let entry = listing
            .entries
            .iter()
            .find(|e| e.name == "dated.txt")
            .unwrap();
        assert_eq!(entry.kind, FileEntryKind::File);
        assert_eq!(entry.size_bytes, Some(0));
        assert!(entry.modified_secs.is_some());
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(entry.modified_secs.unwrap() <= now);
    }
}
