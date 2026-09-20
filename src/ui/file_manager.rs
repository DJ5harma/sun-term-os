use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Paragraph, Row, Table},
};

use crate::app::{AppState, Loadable};
use crate::machine::{FileEntry, FileEntryKind};

use super::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileManagerLayout {
    pub path_row: Rect,
    pub header_row: Rect,
    pub rows_area: Rect,
    pub footer_row: Rect,
}

pub fn layout(inner: Rect) -> FileManagerLayout {
    if inner.height < 4 {
        return FileManagerLayout {
            path_row: inner,
            header_row: Rect::default(),
            rows_area: Rect::default(),
            footer_row: Rect::default(),
        };
    }
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(inner);
    FileManagerLayout {
        path_row: chunks[0],
        header_row: chunks[1],
        rows_area: chunks[2],
        footer_row: chunks[3],
    }
}

pub fn row_rects(rows_area: Rect, entry_count: usize) -> Vec<Rect> {
    let visible = rows_area.height as usize;
    let count = entry_count.min(visible);
    if count == 0 || rows_area.width == 0 || rows_area.height == 0 {
        return Vec::new();
    }
    Layout::vertical(std::iter::repeat_n(Constraint::Length(1), count))
        .split(rows_area)
        .to_vec()
}

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, window_id: u64) {
    let Some(manager) = state.file_manager(window_id) else {
        frame.render_widget(
            Paragraph::new("File manager state is unavailable.")
                .style(Style::default().fg(theme::RED)),
            area,
        );
        return;
    };

    let layout = layout(area);
    let hidden = if manager.show_hidden { "on" } else { "off" };
    frame.render_widget(
        Paragraph::new(format!(
            " {}  ·  hidden: {} ",
            manager.current_path.display(),
            hidden
        ))
        .style(Style::default().fg(theme::MUTED)),
        layout.path_row,
    );

    match &manager.listing {
        Loadable::Loading => {
            frame.render_widget(
                Paragraph::new("Reading directory…").style(theme::muted()),
                layout.rows_area,
            );
        }
        Loadable::Failed(error) => frame.render_widget(
            Paragraph::new(error.as_str()).style(Style::default().fg(theme::RED)),
            layout.rows_area,
        ),
        Loadable::Ready(listing) if listing.entries.is_empty() => frame.render_widget(
            Paragraph::new("This directory is empty.").style(theme::muted()),
            layout.rows_area,
        ),
        Loadable::Ready(listing) => {
            frame.render_widget(
                Paragraph::new(" TYPE   NAME   SIZE   MODIFIED").style(theme::muted()),
                layout.header_row,
            );
            let visible = layout.rows_area.height as usize;
            let rows = listing
                .entries
                .iter()
                .take(visible)
                .enumerate()
                .map(|(index, entry)| {
                    let style = if index == manager.selected_index {
                        theme::active()
                    } else {
                        Style::default().fg(theme::TEXT)
                    };
                    entry_row(entry, style)
                });
            frame.render_widget(
                Table::new(
                    rows,
                    [
                        Constraint::Length(4),
                        Constraint::Min(12),
                        Constraint::Length(10),
                        Constraint::Length(16),
                    ],
                )
                .column_spacing(1),
                layout.rows_area,
            );
            let footer = if listing.entries.len() > visible {
                format!(
                    "{} entries · showing first {} · ↑↓ move · Enter open · Backspace parent · h hidden · r refresh",
                    listing.entries.len(),
                    visible
                )
            } else {
                format!(
                    "{} entries · ↑↓ move · Enter open · Backspace parent · h hidden · r refresh",
                    listing.entries.len()
                )
            };
            frame.render_widget(
                Paragraph::new(footer).style(theme::muted()),
                layout.footer_row,
            );
        }
    }
}

fn entry_row(entry: &FileEntry, style: Style) -> Row<'static> {
    Row::new(vec![
        kind_label(entry.kind),
        entry.name.clone(),
        format_size(entry.size_bytes),
        format_modified(entry.modified_secs),
    ])
    .style(style)
}

fn kind_label(kind: FileEntryKind) -> String {
    match kind {
        FileEntryKind::Directory => "dir".to_owned(),
        FileEntryKind::File => "file".to_owned(),
        FileEntryKind::Symlink => "link".to_owned(),
        FileEntryKind::Other => "other".to_owned(),
    }
}

fn format_size(size_bytes: Option<u64>) -> String {
    match size_bytes {
        Some(bytes) if bytes >= 1_000_000 => format!("{:.1}M", bytes as f64 / 1_000_000.0),
        Some(bytes) if bytes >= 1_000 => format!("{:.1}K", bytes as f64 / 1_000.0),
        Some(bytes) => format!("{}B", bytes),
        None => "—".to_owned(),
    }
}

fn format_modified(modified_secs: Option<u64>) -> String {
    let Some(secs) = modified_secs else {
        return "—".to_owned();
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(secs);
    let age = now.saturating_sub(secs);
    if age >= 86_400 {
        format!("{}d ago", age / 86_400)
    } else if age >= 3_600 {
        format!("{}h ago", age / 3_600)
    } else if age >= 60 {
        format!("{}m ago", age / 60)
    } else {
        "recent".to_owned()
    }
}

pub fn inner_content_area(window_area: Rect) -> Rect {
    Block::default().borders(Borders::ALL).inner(window_area)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn row_rects_match_visible_entry_count() {
        let area = Rect::new(0, 0, 40, 12);
        let layout = layout(area);
        let visible = layout.rows_area.height as usize;
        let rects = row_rects(layout.rows_area, 100);
        assert_eq!(rects.len(), visible);
    }
}
