use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::ApplicationEntry;
use crate::machine::CapabilityError;

pub fn desktop_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
        paths.push(PathBuf::from(data_home).join("applications"));
    } else if let Ok(home) = std::env::var("HOME") {
        paths.push(PathBuf::from(home).join(".local/share/applications"));
    }
    if let Ok(dirs) = std::env::var("XDG_DATA_DIRS") {
        for dir in dirs.split(':') {
            paths.push(PathBuf::from(dir).join("applications"));
        }
    } else {
        paths.push(PathBuf::from("/usr/share/applications"));
        paths.push(PathBuf::from("/usr/local/share/applications"));
    }
    paths
}

pub fn scan_desktop_dirs(paths: &[PathBuf]) -> Result<Vec<ApplicationEntry>, CapabilityError> {
    let mut entries = Vec::new();
    let mut seen = HashSet::new();
    for dir in paths {
        if !dir.is_dir() {
            continue;
        }
        let read_dir =
            std::fs::read_dir(dir).map_err(|error| CapabilityError::Failed(error.to_string()))?;
        for item in read_dir.flatten() {
            let path = item.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("desktop") {
                continue;
            }
            let text = std::fs::read_to_string(&path)
                .map_err(|error| CapabilityError::Failed(error.to_string()))?;
            if let Some(entry) = parse_desktop_text(&path, &text)
                && seen.insert(entry.id.clone())
            {
                entries.push(entry);
            }
        }
    }
    entries.sort_by_key(|entry| entry.name.to_lowercase());
    Ok(entries)
}

pub fn parse_desktop_text(path: &Path, text: &str) -> Option<ApplicationEntry> {
    let id = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("app")
        .to_owned();
    parse_desktop_fields(&id, text)
}

pub fn parse_desktop_fields(id: &str, text: &str) -> Option<ApplicationEntry> {
    let mut name = None;
    let mut comment = None;
    let mut exec = None;
    let mut terminal = false;
    let mut no_display = false;
    for line in text.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("Name=") {
            name = Some(value.trim().to_owned());
        } else if let Some(value) = line.strip_prefix("Comment=") {
            comment = Some(value.trim().to_owned());
        } else if let Some(value) = line.strip_prefix("Exec=") {
            exec = Some(strip_exec_field_codes(value.trim()));
        } else if line == "Terminal=true" {
            terminal = true;
        } else if line == "NoDisplay=true" || line == "Hidden=true" {
            no_display = true;
        }
    }
    if no_display {
        return None;
    }
    let name = name?;
    let exec = exec?;
    if exec.is_empty() {
        return None;
    }
    Some(ApplicationEntry {
        id: id.to_owned(),
        name,
        detail: comment.unwrap_or_else(|| exec.clone()),
        exec,
        launch_in_terminal: terminal,
    })
}

pub fn parse_remote_discover_output(text: &str) -> Vec<ApplicationEntry> {
    let mut entries = Vec::new();
    let mut seen = HashSet::new();
    let mut current_path: Option<String> = None;
    let mut block = String::new();
    for line in text.lines() {
        if let Some(path) = line.strip_prefix("@PATH@") {
            if let Some(path) = current_path.as_deref() {
                flush_remote_block(path, &block, &mut entries, &mut seen);
            }
            current_path = Some(path.to_owned());
            block.clear();
            continue;
        }
        if line == "@END@" {
            if let Some(path) = current_path.as_deref() {
                flush_remote_block(path, &block, &mut entries, &mut seen);
            }
            block.clear();
            continue;
        }
        if current_path.is_some() {
            block.push_str(line);
            block.push('\n');
        }
    }
    if let Some(path) = current_path.as_deref() {
        flush_remote_block(path, &block, &mut entries, &mut seen);
    }
    entries.sort_by_key(|entry| entry.name.to_lowercase());
    entries
}

fn flush_remote_block(
    path: &str,
    block: &str,
    entries: &mut Vec<ApplicationEntry>,
    seen: &mut HashSet<String>,
) {
    if block.is_empty() {
        return;
    }
    let id = Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("app");
    if let Some(entry) = parse_desktop_fields(id, block)
        && seen.insert(entry.id.clone())
    {
        entries.push(entry);
    }
}

fn strip_exec_field_codes(exec: &str) -> String {
    exec.split_whitespace()
        .filter(|token| !token.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_remote_block_format() {
        let output = "@PATH@/usr/share/applications/demo.desktop\n\
Name=Demo\nComment=Example\nExec=demo %f\nTerminal=false\n@END@\n";
        let entries = parse_remote_discover_output(output);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "demo");
        assert_eq!(entries[0].name, "Demo");
        assert_eq!(entries[0].exec, "demo");
    }
}
