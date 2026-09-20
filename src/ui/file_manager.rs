use crate::actions::{Action, FileManagerAction};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

/// Gap between icon / name / size / modified columns (must match header and rows).
const LIST_COLUMN_SPACING: u16 = 2;
const ICON_COL_WIDTH: u16 = 2;
const SIZE_COL_WIDTH: u16 = 10;
const MODIFIED_COL_WIDTH: u16 = 19;

use crate::{
    app::{
        file_manager::{DisplayRowKind, SortColumn, display_row_count, display_row_kind},
        state::{
            AppState, FileManagerDialog, FileManagerFocus, FileManagerState, FileSort, Loadable,
        },
    },
    machine::FileEntryKind,
};

use super::{
    interaction::{InteractionLayer, InteractionMap},
    theme,
};

#[derive(Debug, Clone, Copy)]
pub struct FileManagerLayout {
    pub toolbar: Rect,
    pub places: Rect,
    pub list_header: Rect,
    pub list_rows: Rect,
    pub status: Rect,
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

fn places_panel_inner(area: Rect) -> Rect {
    places_panel_block().inner(area)
}

fn places_panel_block() -> Block<'static> {
    Block::default()
        .title(" Places ")
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(theme::surface_alt()))
}

fn list_column_constraints() -> [Constraint; 4] {
    [
        Constraint::Length(ICON_COL_WIDTH),
        Constraint::Min(8),
        Constraint::Length(SIZE_COL_WIDTH),
        Constraint::Length(MODIFIED_COL_WIDTH),
    ]
}

fn list_column_rects(row: Rect) -> [Rect; 4] {
    let chunks = Layout::horizontal(list_column_constraints())
        .spacing(LIST_COLUMN_SPACING)
        .split(row);
    [chunks[0], chunks[1], chunks[2], chunks[3]]
}

fn list_table<'a, I>(rows: I) -> Table<'a>
where
    I: IntoIterator<Item = Row<'a>>,
{
    Table::new(rows, list_column_constraints()).column_spacing(LIST_COLUMN_SPACING)
}

fn cell_right(text: String) -> Cell<'static> {
    Cell::from(Line::from(text).alignment(Alignment::Right))
}

fn cell_left(text: impl Into<String>) -> Cell<'static> {
    Cell::from(text.into())
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    window_id: u64,
    interactions: &mut InteractionMap,
) {
    let Some(manager) = state.file_manager(window_id) else {
        frame.render_widget(
            Paragraph::new("File manager state is unavailable.")
                .style(Style::default().fg(theme::red())),
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
            Paragraph::new(format!("  {}", error)).style(Style::default().fg(theme::red())),
            layout.list_rows,
        ),
        Loadable::Ready(listing) => {
            render_list_header(frame, layout.list_header, manager.sort);
            render_list(frame, layout.list_rows, manager, listing);
        }
    }
    register_pointer_targets(interactions, window_id, &layout, manager);
}

fn register_pointer_targets(
    interactions: &mut InteractionMap,
    window_id: u64,
    layout: &FileManagerLayout,
    manager: &FileManagerState,
) {
    if layout.toolbar.width >= 9 {
        let layer = InteractionLayer::Content;
        interactions.register(
            layer,
            Rect::new(layout.toolbar.x, layout.toolbar.y, 3, 1),
            Action::FileManager(FileManagerAction::FileManagerGoBack),
        );
        interactions.register(
            layer,
            Rect::new(layout.toolbar.x + 3, layout.toolbar.y, 3, 1),
            Action::FileManager(FileManagerAction::FileManagerGoUp),
        );
        interactions.register(
            layer,
            Rect::new(layout.toolbar.x + 6, layout.toolbar.y, 3, 1),
            Action::FileManager(FileManagerAction::FileManagerGoHome),
        );
    }

    let places_inner = places_panel_inner(layout.places);
    let place_count = manager.places.len();
    if place_count > 0 && places_inner.height > 0 {
        let visible_places = place_count.min(places_inner.height as usize);
        let rows = Layout::vertical(std::iter::repeat_n(Constraint::Length(1), visible_places))
            .split(places_inner);
        for (index, rect) in rows.iter().enumerate() {
            interactions.register(
                InteractionLayer::Content,
                *rect,
                Action::FileManager(FileManagerAction::SelectFileManagerPlace(window_id, index)),
            );
        }
    }

    if layout.list_header.width > 20 {
        let cols = list_column_rects(layout.list_header);
        let layer = InteractionLayer::Content;
        interactions.register(
            layer,
            cols[1],
            Action::FileManager(FileManagerAction::FileManagerSetSort(SortColumn::Name)),
        );
        interactions.register(
            layer,
            cols[2],
            Action::FileManager(FileManagerAction::FileManagerSetSort(SortColumn::Size)),
        );
        interactions.register(
            layer,
            cols[3],
            Action::FileManager(FileManagerAction::FileManagerSetSort(SortColumn::Modified)),
        );
    }

    if let Loadable::Ready(listing) = &manager.listing {
        let total = display_row_count(listing, &manager.current_path);
        let visible = layout.list_rows.height.max(1) as usize;
        let start = manager.scroll_offset;
        let count = total.saturating_sub(start).min(visible);
        let rows = Layout::vertical(std::iter::repeat_n(Constraint::Length(1), count))
            .split(layout.list_rows);
        for (offset, rect) in rows.iter().enumerate() {
            interactions.register(
                InteractionLayer::Content,
                *rect,
                Action::FileManager(FileManagerAction::SelectFileManagerRow(
                    window_id,
                    start + offset,
                )),
            );
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
        Span::styled(" ← ", Style::default().fg(theme::blue())),
        Span::styled(" ↑ ", Style::default().fg(theme::blue())),
        Span::styled(" ⌂ ", Style::default().fg(theme::blue())),
        Span::styled(breadcrumb, Style::default().fg(theme::text())),
        Span::styled(format!("  {}  R", hidden), theme::muted()),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn render_places(frame: &mut Frame, area: Rect, manager: &FileManagerState) {
    let block = places_panel_block();
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
            Style::default().fg(theme::text())
        };
        Row::new(vec![Cell::from(format!("  {}", place.label))]).style(style)
    });
    frame.render_widget(Table::new(rows, [Constraint::Percentage(100)]), inner);
}

fn render_list_header(frame: &mut Frame, area: Rect, sort: FileSort) {
    let header = Row::new(vec![
        Cell::from(""),
        header_cell("Name", SortColumn::Name, sort, Alignment::Left),
        header_cell("Size", SortColumn::Size, sort, Alignment::Right),
        header_cell("Modified", SortColumn::Modified, sort, Alignment::Right),
    ]);
    frame.render_widget(list_table([header]).style(theme::muted()), area);
}

fn header_cell(
    title: &str,
    column: SortColumn,
    sort: FileSort,
    alignment: Alignment,
) -> Cell<'static> {
    let arrow = if sort.column == column {
        if sort.ascending { " ▲" } else { " ▼" }
    } else {
        ""
    };
    let text = format!("{title}{arrow}");
    let style = if sort.column == column {
        theme::active()
    } else {
        theme::muted()
    };
    Cell::from(Line::from(text).alignment(alignment).style(style))
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
            Style::default().fg(theme::text())
        };
        row_for_index(&manager.current_path, listing, index, style)
    });
    frame.render_widget(list_table(rows), area);
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
            cell_left(".."),
            cell_right(String::new()),
            cell_right(String::new()),
        ])
        .style(style),
        Some(DisplayRowKind::Entry) => {
            let offset = usize::from(path.parent().is_some_and(|parent| parent != path));
            let entry = listing.entries.get(index - offset).expect("entry index");
            Row::new(vec![
                Cell::from(icon(entry.kind)),
                cell_left(entry.name.clone()),
                cell_right(format_size(entry.size_bytes, entry.kind)),
                cell_right(format_modified(entry.modified_secs)),
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
    let text = match &manager.dialog {
        FileManagerDialog::DeleteConfirm { label, .. } => {
            format!("  Trash/delete {label}?  y confirm · n/Esc cancel")
        }
        FileManagerDialog::Rename { input, .. } => {
            format!("  Rename to: {input}▌  Enter save · Esc cancel")
        }
        FileManagerDialog::Create { kind, input } => {
            let label = match kind {
                crate::app::file_manager::CreateKind::File => "file",
                crate::app::file_manager::CreateKind::Directory => "folder",
            };
            format!("  New {label}: {input}▌  Enter create · Esc cancel")
        }
        FileManagerDialog::GoToPath { input } => {
            format!("  Go to: {input}▌  Enter · Esc cancel")
        }
        FileManagerDialog::None => match &manager.listing {
            Loadable::Ready(listing) => format!(
                "  {} items · : path · a/A new · d trash · Shift+R rename · o term in folder · e edit · Shift+O system",
                listing.entries.len() + usize::from(listing.path.parent().is_some()),
            ),
            Loadable::Loading => "  Reading…".to_owned(),
            Loadable::Failed(_) => "  Could not read this folder".to_owned(),
        },
    };
    frame.render_widget(Paragraph::new(text).style(theme::muted()), area);
}

fn format_size(size_bytes: Option<u64>, kind: FileEntryKind) -> String {
    if kind == FileEntryKind::Directory {
        return "—".to_owned();
    }
    match size_bytes {
        Some(bytes) if bytes >= 1_048_576 => format!("{:.1} MB", bytes as f64 / 1_048_576.0),
        Some(bytes) if bytes >= 1024 => format!("{:.1} KB", bytes as f64 / 1024.0),
        Some(bytes) => format!("{} B", bytes),
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
    fn places_sidebar_hits_use_inner_below_title() {
        let area = Rect::new(4, 6, 18, 8);
        let inner = places_panel_inner(area);
        assert!(
            inner.y > area.y,
            "place rows must be below the Places title"
        );
        assert_eq!(inner.x, area.x);
    }

    #[test]
    fn layout_splits_toolbar_places_and_list() {
        let layout = layout(Rect::new(0, 0, 80, 20));
        assert_eq!(layout.toolbar.height, 1);
        assert_eq!(layout.status.height, 1);
        assert!(layout.places.width >= 18);
        assert!(layout.list_rows.height >= 1);
    }
}
