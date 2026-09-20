//! File manager domain logic (sorting, places, display rows, scroll). No I/O here.

use std::path::{Path, PathBuf};

/// Escape a path for use inside single-quoted shell words.
pub fn shell_single_quoted(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
}

use crate::machine::{DirectoryListing, FileEntry, FileEntryKind};

use super::Loadable;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileManagerFocus {
    Places,
    List,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Name,
    Size,
    Modified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileSort {
    pub column: SortColumn,
    pub ascending: bool,
}

impl Default for FileSort {
    fn default() -> Self {
        Self {
            column: SortColumn::Name,
            ascending: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    pub label: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayRowKind {
    Parent,
    Entry,
}

pub fn standard_places() -> Vec<Place> {
    let home = std::env::var("HOME")
        .ok()
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
        .unwrap_or_else(|| PathBuf::from("/"));
    let mut places = vec![
        Place {
            label: "Home".to_owned(),
            path: home.clone(),
        },
        Place {
            label: "Root".to_owned(),
            path: PathBuf::from("/"),
        },
    ];
    for (label, segment) in [("Desktop", "Desktop"), ("Documents", "Documents")] {
        let path = home.join(segment);
        if path.is_dir() {
            places.push(Place {
                label: label.to_owned(),
                path,
            });
        }
    }
    places
}

pub fn sort_entries(entries: &mut [FileEntry], sort: FileSort) {
    entries.sort_by(|left, right| {
        let left_dir = left.kind == FileEntryKind::Directory;
        let right_dir = right.kind == FileEntryKind::Directory;
        let ordering = right_dir.cmp(&left_dir).then_with(|| match sort.column {
            SortColumn::Name => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
            SortColumn::Size => left
                .size_bytes
                .unwrap_or(0)
                .cmp(&right.size_bytes.unwrap_or(0)),
            SortColumn::Modified => left
                .modified_secs
                .unwrap_or(0)
                .cmp(&right.modified_secs.unwrap_or(0)),
        });
        if sort.ascending {
            ordering
        } else {
            ordering.reverse()
        }
    });
}

pub fn display_row_count(listing: &DirectoryListing, path: &Path) -> usize {
    let parent = usize::from(path.parent().is_some_and(|parent| parent != path));
    parent + listing.entries.len()
}

pub fn display_row_kind(
    path: &Path,
    listing: &DirectoryListing,
    index: usize,
) -> Option<DisplayRowKind> {
    if index == 0 && path.parent().is_some_and(|parent| parent != path) {
        return Some(DisplayRowKind::Parent);
    }
    let offset = usize::from(path.parent().is_some_and(|parent| parent != path));
    if index >= offset + listing.entries.len() {
        return None;
    }
    Some(DisplayRowKind::Entry)
}

pub fn display_entry<'a>(
    path: &Path,
    listing: &'a DirectoryListing,
    index: usize,
) -> Option<&'a FileEntry> {
    match display_row_kind(path, listing, index)? {
        DisplayRowKind::Parent => None,
        DisplayRowKind::Entry => {
            let offset = usize::from(path.parent().is_some_and(|parent| parent != path));
            listing.entries.get(index - offset)
        }
    }
}

pub fn ensure_selection_visible(
    selected: &mut usize,
    scroll: &mut usize,
    visible_rows: usize,
    total_rows: usize,
) {
    if total_rows == 0 {
        *selected = 0;
        *scroll = 0;
        return;
    }
    *selected = (*selected).min(total_rows - 1);
    if *selected < *scroll {
        *scroll = *selected;
    }
    if visible_rows == 0 {
        return;
    }
    if *selected >= *scroll + visible_rows {
        *scroll = selected.saturating_sub(visible_rows - 1);
    }
    let max_scroll = total_rows.saturating_sub(visible_rows);
    *scroll = (*scroll).min(max_scroll);
}

pub fn apply_sorted_listing(listing: DirectoryListing, sort: FileSort) -> DirectoryListing {
    let mut entries = listing.entries;
    sort_entries(&mut entries, sort);
    DirectoryListing {
        path: listing.path,
        entries,
    }
}

pub fn clamp_listing_selection(
    listing: &Loadable<DirectoryListing>,
    path: &Path,
    selected_index: &mut usize,
    scroll_offset: &mut usize,
    visible_rows: usize,
) {
    let total = match listing {
        Loadable::Ready(listing) => display_row_count(listing, path),
        _ => 0,
    };
    ensure_selection_visible(selected_index, scroll_offset, visible_rows, total);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_row_is_first_when_not_at_root() {
        let listing = DirectoryListing {
            path: PathBuf::from("/tmp/nested"),
            entries: vec![FileEntry {
                name: "a".to_owned(),
                kind: FileEntryKind::File,
                size_bytes: Some(1),
                modified_secs: None,
            }],
        };
        assert_eq!(display_row_count(&listing, &listing.path), 2);
        assert_eq!(
            display_row_kind(&listing.path, &listing, 0),
            Some(DisplayRowKind::Parent)
        );
        assert_eq!(
            display_entry(&listing.path, &listing, 1).map(|e| e.name.as_str()),
            Some("a")
        );
    }

    #[test]
    fn scroll_follows_selection() {
        let mut selected = 10;
        let mut scroll = 0;
        ensure_selection_visible(&mut selected, &mut scroll, 5, 20);
        assert_eq!(scroll, 6);
    }
}
