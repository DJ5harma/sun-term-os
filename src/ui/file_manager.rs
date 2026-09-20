use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::{
    app::{
        file_manager::{DisplayRowKind, SortColumn, display_row_count, display_row_kind},
        state::{AppState, FileManagerFocus, FileManagerState, FileSort, Loadable},
    },
    machine::FileEntryKind,
};

use super::theme;

#[derive(Debug, Clone, Copy)]
pub struct FileManagerLayout {
    pub toolbar: Rect,
    pub places: Rect,
    pub list_header: Rect,
    pub list_rows: Rect,
    pub status: Rect,
}

#[derive(Debug, Clone, Default)]
pub struct FileManagerHitTargets {
    pub back: Rect,
    pub up: Rect,
    pub home: Rect,
    pub place_rows: Vec<(usize, Rect)>,
    pub list_rows: Vec<(usize, Rect)>,
    pub sort_name: Rect,
    pub sort_size: Rect,
    pub sort_modified: Rect,
}

pub fn layout(inner: Rect) -> FileManagerLayout {
    if inner.height < 6 {
        return FileManagerLayout {
            toolbar: inner,
            places: Rect::default(),
            list_header: Rect::default(),
            list_rows: Rect::default(),
            status: Rect::default(),
        };
    }
    let vertical = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(4),
        Constraint::Length(1),
    ])
    .split(inner);
    let body = Layout::horizontal([Constraint::Length(18), Constraint::Min(10)]).split(vertical[1]);
    let list = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(body[1]);
    FileManagerLayout {
        toolbar: vertical[0],
        places: body[0],
        list_header: list[0],
        list_rows: list[1],
        status: vertical[2],
    }
}

pub fn hit_targets(
    layout: &FileManagerLayout,
    manager: &FileManagerState,
) -> FileManagerHitTargets {
    let mut targets = FileManagerHitTargets::default();
    if layout.toolbar.width >= 9 {
        targets.back = Rect::new(layout.toolbar.x, layout.toolbar.y, 3, 1);
        targets.up = Rect::new(layout.toolbar.x + 3, layout.toolbar.y, 3, 1);
        targets.home = Rect::new(layout.toolbar.x + 6, layout.toolbar.y, 3, 1);
    }
    let place_count = manager.places.len();
    if place_count > 0 && layout.places.height > 0 {
        targets.place_rows = Layout::vertical(std::iter::repeat_n(
            Constraint::Length(1),
            place_count.min(layout.places.height as usize),
        ))
        .split(layout.places)
        .iter()
        .enumerate()
        .map(|(index, rect)| (index, *rect))
        .collect();
    }
    if let Loadable::Ready(listing) = &manager.listing {
        let total = display_row_count(listing, &manager.current_path);
        let visible = layout.list_rows.height.max(1) as usize;
        let start = manager.scroll_offset;
        let count = total.saturating_sub(start).min(visible);
        targets.list_rows = Layout::vertical(std::iter::repeat_n(Constraint::Length(1), count))
            .split(layout.list_rows)
            .iter()
            .enumerate()
            .map(|(offset, rect)| (start + offset, *rect))
            .collect();
    }
    if layout.list_header.width > 20 {
        targets.sort_name = Rect::new(layout.list_header.x + 2, layout.list_header.y, 12, 1);
        targets.sort_size = Rect::new(
            layout.list_header.x + layout.list_header.width / 2,
            layout.list_header.y,
            8,
            1,
        );
        targets.sort_modified = Rect::new(
            layout.list_header.x + layout.list_header.width - 12,
            layout.list_header.y,
            12,
            1,
        );
    }
    targets
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

    render_toolbar(frame, layout.toolbar, manager);
    render_places(frame, layout.places, manager);
    render_status(frame, layout.status, manager);

    match &manager.listing {
        Loadable::Loading => frame.render_widget(
            Paragraph::new("  Loading folder contents…").style(theme::muted()),
            layout.list_rows,
        ),
        Loadable::Failed(error) => frame.render_widget(
            Paragraph::new(format!("  {}", error)).style(Style::default().fg(theme::RED)),
            layout.list_rows,
        ),
        Loadable::Ready(listing) => {
            render_list_header(frame, layout.list_header, manager.sort);
            render_list(frame, layout.list_rows, manager, listing);
        }
    }
}

fn render_toolbar(frame: &mut Frame, area: Rect, manager: &FileManagerState) {
    let hidden = if manager.show_hidden { "H*" } else { "H" };
    let breadcrumb = breadcrumb_text(
        &manager.current_path,
        area.width.saturating_sub(14) as usize,
    );
    let line = Line::from(vec![
        Span::styled(" ← ", Style::default().fg(theme::BLUE)),
        Span::styled(" ↑ ", Style::default().fg(theme::BLUE)),
        Span::styled(" ⌂ ", Style::default().fg(theme::BLUE)),
        Span::styled(breadcrumb, Style::default().fg(theme::TEXT)),
        Span::styled(format!("  {}  R", hidden), theme::muted()),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn render_places(frame: &mut Frame, area: Rect, manager: &FileManagerState) {
    let block = Block::default()
        .title(" Places ")
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(theme::SURFACE_ALT));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if manager.places.is_empty() {
        return;
    }
    let rows = manager.places.iter().enumerate().map(|(index, place)| {
        let style = if manager.focus == FileManagerFocus::Places && index == manager.selected_place
        {
            theme::active()
        } else {
            Style::default().fg(theme::TEXT)
        };
        Row::new(vec![Cell::from(format!("  {}", place.label))]).style(style)
    });
    frame.render_widget(Table::new(rows, [Constraint::Percentage(100)]), inner);
}

fn render_list_header(frame: &mut Frame, area: Rect, sort: FileSort) {
    let line = Line::from(vec![
        sort_label("Name", SortColumn::Name, sort),
        Span::raw("  "),
        sort_label("Size", SortColumn::Size, sort),
        Span::raw("    "),
        sort_label("Modified", SortColumn::Modified, sort),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn sort_label(title: &str, column: SortColumn, sort: FileSort) -> Span<'static> {
    let arrow = if sort.column == column {
        if sort.ascending { " ▲" } else { " ▼" }
    } else {
        ""
    };
    let text = format!("{}{}", title, arrow);
    let style = if sort.column == column {
        theme::active()
    } else {
        theme::muted()
    };
    Span::styled(text, style)
}

fn render_list(
    frame: &mut Frame,
    area: Rect,
    manager: &FileManagerState,
    listing: &crate::machine::DirectoryListing,
) {
    let total = display_row_count(listing, &manager.current_path);
    if total == 0 {
        frame.render_widget(Paragraph::new("  Empty folder").style(theme::muted()), area);
        return;
    }
    let visible = area.height.max(1) as usize;
    let start = manager.scroll_offset;
    let end = (start + visible).min(total);
    let rows = (start..end).map(|index| {
        let selected = manager.focus == FileManagerFocus::List && index == manager.selected_index;
        let style = if selected {
            theme::active()
        } else {
            Style::default().fg(theme::TEXT)
        };
        row_for_index(&manager.current_path, listing, index, style)
    });
    frame.render_widget(
        Table::new(
            rows,
            [
                Constraint::Length(2),
                Constraint::Min(10),
                Constraint::Length(9),
                Constraint::Length(16),
            ],
        )
        .column_spacing(1),
        area,
    );
}

fn row_for_index(
    path: &std::path::Path,
    listing: &crate::machine::DirectoryListing,
    index: usize,
    style: Style,
) -> Row<'static> {
    match display_row_kind(path, listing, index) {
        Some(DisplayRowKind::Parent) => Row::new(vec![
            Cell::from("↰"),
            Cell::from(".."),
            Cell::from(""),
            Cell::from(""),
        ])
        .style(style),
        Some(DisplayRowKind::Entry) => {
            let offset = usize::from(path.parent().is_some_and(|parent| parent != path));
            let entry = listing.entries.get(index - offset).expect("entry index");
            Row::new(vec![
                Cell::from(icon(entry.kind)),
                Cell::from(entry.name.clone()),
                Cell::from(format_size(entry.size_bytes, entry.kind)),
                Cell::from(format_modified(entry.modified_secs)),
            ])
            .style(style)
        }
        None => Row::new(vec![
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ]),
    }
}

fn icon(kind: FileEntryKind) -> &'static str {
    match kind {
        FileEntryKind::Directory => "▸",
        FileEntryKind::File => "·",
        FileEntryKind::Symlink => "⇢",
        FileEntryKind::Other => "?",
    }
}

fn breadcrumb_text(path: &std::path::Path, max_len: usize) -> String {
    let text = if path.as_os_str().is_empty() {
        "/".to_owned()
    } else {
        path.display().to_string()
    };
    if text.len() <= max_len {
        return format!(" {text} ");
    }
    format!(" …{} ", &text[text.len() - max_len.saturating_sub(4)..])
}

fn render_status(frame: &mut Frame, area: Rect, manager: &FileManagerState) {
    let text = match &manager.listing {
        Loadable::Ready(listing) => {
            let folders = listing
                .entries
                .iter()
                .filter(|e| e.kind == FileEntryKind::Directory)
                .count();
            let files = listing.entries.len() - folders;
            format!(
                "  {} items ({} folders, {} files) · dbl-click open · Tab places · Enter · PgUp/PgDn scroll",
                listing.entries.len() + usize::from(listing.path.parent().is_some()),
                folders,
                files
            )
        }
        Loadable::Loading => "  Reading…".to_owned(),
        Loadable::Failed(_) => "  Could not read this folder".to_owned(),
    };
    frame.render_widget(Paragraph::new(text).style(theme::muted()), area);
}

fn format_size(size_bytes: Option<u64>, kind: FileEntryKind) -> String {
    if kind == FileEntryKind::Directory {
        return "—".to_owned();
    }
    match size_bytes {
        Some(bytes) if bytes >= 1_048_576 => format!("{:>7.1} MB", bytes as f64 / 1_048_576.0),
        Some(bytes) if bytes >= 1024 => format!("{:>7.1} KB", bytes as f64 / 1024.0),
        Some(bytes) => format!("{:>7} B", bytes),
        None => "—".to_owned(),
    }
}

fn format_modified(modified_secs: Option<u64>) -> String {
    let Some(secs) = modified_secs else {
        return "—".to_owned();
    };
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let hours = rem / 3_600;
    let minutes = (rem % 3_600) / 60;
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        1970 + days / 365,
        (days % 12) + 1,
        (days % 28) + 1,
        hours,
        minutes
    )
}

pub fn sync_visible_rows(manager: &mut FileManagerState, list_rows: Rect) {
    manager.visible_rows = list_rows.height.max(1) as usize;
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn layout_splits_toolbar_places_and_list() {
        let layout = layout(Rect::new(0, 0, 80, 20));
        assert_eq!(layout.toolbar.height, 1);
        assert_eq!(layout.status.height, 1);
        assert!(layout.places.width >= 18);
        assert!(layout.list_rows.height >= 1);
    }
}
