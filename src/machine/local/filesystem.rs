use std::path::{Path, PathBuf};

use async_trait::async_trait;

use super::super::{
    CapabilityError, DirectoryListing, FileEntry, FileEntryKind, FilesystemProvider, RemoveOutcome,
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

    async fn remove_path(&self, path: &Path) -> Result<RemoveOutcome, CapabilityError> {
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || remove_path_blocking(&path))
            .await
            .map_err(|error| CapabilityError::Failed(error.to_string()))?
    }

    async fn create_file(&self, path: &Path) -> Result<(), CapabilityError> {
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || create_file_blocking(&path))
            .await
            .map_err(|error| CapabilityError::Failed(error.to_string()))?
    }

    async fn create_directory(&self, path: &Path) -> Result<(), CapabilityError> {
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || create_directory_blocking(&path))
            .await
            .map_err(|error| CapabilityError::Failed(error.to_string()))?
    }

    async fn rename_path(&self, from: &Path, to: &Path) -> Result<(), CapabilityError> {
        let from = from.to_path_buf();
        let to = to.to_path_buf();
        tokio::task::spawn_blocking(move || rename_path_blocking(&from, &to))
            .await
            .map_err(|error| CapabilityError::Failed(error.to_string()))?
    }
}

fn remove_path_blocking(path: &Path) -> Result<RemoveOutcome, CapabilityError> {
    if !path.exists() {
        return Err(CapabilityError::Failed(format!(
            "Path does not exist: {}",
            path.display()
        )));
    }
    if trash::delete(path).is_ok() {
        return Ok(RemoveOutcome::MovedToTrash);
    }
    delete_path_blocking(path)?;
    Ok(RemoveOutcome::DeletedPermanently)
}

fn create_file_blocking(path: &Path) -> Result<(), CapabilityError> {
    if path.exists() {
        return Err(CapabilityError::Failed(format!(
            "Already exists: {}",
            path.display()
        )));
    }
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        return Err(CapabilityError::Failed(format!(
            "Parent directory missing: {}",
            parent.display()
        )));
    }
    std::fs::write(path, []).map_err(map_io_error)
}

fn create_directory_blocking(path: &Path) -> Result<(), CapabilityError> {
    if path.exists() {
        return Err(CapabilityError::Failed(format!(
            "Already exists: {}",
            path.display()
        )));
    }
    std::fs::create_dir(path).map_err(map_io_error)
}

fn delete_path_blocking(path: &Path) -> Result<(), CapabilityError> {
    if !path.exists() {
        return Err(CapabilityError::Failed(format!(
            "Path does not exist: {}",
            path.display()
        )));
    }
    if path.is_dir() {
        std::fs::remove_dir_all(path).map_err(map_io_error)
    } else {
        std::fs::remove_file(path).map_err(map_io_error)
    }
}

fn rename_path_blocking(from: &Path, to: &Path) -> Result<(), CapabilityError> {
    if !from.exists() {
        return Err(CapabilityError::Failed(format!(
            "Path does not exist: {}",
            from.display()
        )));
    }
    if to.exists() {
        return Err(CapabilityError::Failed(format!(
            "Destination already exists: {}",
            to.display()
        )));
    }
    if let Some(parent) = to.parent()
        && !parent.exists()
    {
        return Err(CapabilityError::Failed(format!(
            "Parent directory missing: {}",
            parent.display()
        )));
    }
    std::fs::rename(from, to).map_err(map_io_error)
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
    fn create_file_and_directory() {
        let root = tempfile::tempdir().expect("tempdir");
        let file = root.path().join("new.txt");
        create_file_blocking(&file).expect("create file");
        assert!(file.is_file());

        let dir = root.path().join("subdir");
        create_directory_blocking(&dir).expect("create dir");
        assert!(dir.is_dir());
    }

    #[test]
    fn delete_and_rename_paths() {
        let root = tempfile::tempdir().expect("tempdir");
        let file = root.path().join("old.txt");
        touch(&file);
        remove_path_blocking(&file).expect("delete file");
        assert!(!file.exists());

        let folder = root.path().join("dir");
        fs::create_dir(&folder).expect("mkdir");
        fs::write(folder.join("nested.txt"), b"x").expect("write");
        delete_path_blocking(&folder).expect("delete dir");
        assert!(!folder.exists());

        let rename_src = root.path().join("a.txt");
        touch(&rename_src);
        let rename_dst = root.path().join("b.txt");
        rename_path_blocking(&rename_src, &rename_dst).expect("rename");
        assert!(!rename_src.exists());
        assert!(rename_dst.is_file());
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
