use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    app::{
        AppState,
        home_screen::{HomeEntry, HomeScreenMode, entries},
    },
    apps::open_action,
    input::{LAUNCHER_SHORTCUT_HINT, SHOW_DESKTOP_HINT, WINDOW_FOCUS_HINT, WORKSPACE_HINT},
};

use super::{
    interaction::{InteractionLayer, InteractionMap},
    theme,
};

const TILE_MIN_WIDTH: u16 = 24;
const TILE_HEIGHT: u16 = 4;
const TILE_GAP: u16 = 1;

pub struct HomeLayout {
    pub columns: usize,
    pub tile_rows_visible: usize,
    pub total_tile_rows: usize,
    pub grid: Rect,
}

pub fn layout(
    area: Rect,
    entry_count: usize,
    mode: HomeScreenMode,
    show_desktop: bool,
) -> HomeLayout {
    let header = 1u16;
    let footer = 1u16;
    let banner =
        (show_desktop || matches!(mode, HomeScreenMode::WindowsInBackground { .. })) as u16;
    let grid_top = area.y + header + banner;
    let grid_height = area.height.saturating_sub(header + footer + banner);
    let columns = column_count(area.width, entry_count);
    let total_tile_rows = if entry_count == 0 {
        0
    } else {
        entry_count.div_ceil(columns)
    };
    let row_stride = TILE_HEIGHT + TILE_GAP;
    let tile_rows_visible = grid_height.checked_div(row_stride).unwrap_or(1).max(1) as usize;
    let grid = Rect {
        x: area.x,
        y: grid_top,
        width: area.width,
        height: grid_height,
    };
    HomeLayout {
        columns,
        tile_rows_visible,
        total_tile_rows,
        grid,
    }
}

fn column_count(area_width: u16, entry_count: usize) -> usize {
    if entry_count == 0 || area_width < TILE_MIN_WIDTH {
        return 1;
    }
    let stride = TILE_MIN_WIDTH + TILE_GAP;
    let max_cols = (area_width / stride).max(1) as usize;
    max_cols.min(entry_count).max(1)
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    mode: HomeScreenMode,
    interactions: &mut InteractionMap,
) {
    let list = entries(&state.config);
    let show_desktop = state.current_workspace().show_desktop;
    let layout = layout(area, list.len(), mode, show_desktop);
    let home = state.home_screen();

    let host = state
        .focused_window()
        .map(|window| state.host_label_for(&window.machine_id))
        .unwrap_or_else(|| state.host_label());
    let workspace = state.active_workspace + 1;
    let header = Line::from(vec![
        Span::styled(" sun-term-os ", theme::active()),
        Span::styled(
            format!("· ws {workspace} · {host} "),
            Style::default().fg(theme::muted_color()),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(header),
        Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        },
    );

    let banner_y = area.y + 1;
    if state.current_workspace().show_desktop {
        let banner = format!(
            " Show desktop — {total} window(s) hidden · ⌂ or {SHOW_DESKTOP_HINT} to restore ",
            total = state.current_workspace().windows.len()
        );
        frame.render_widget(
            Paragraph::new(banner).style(Style::default().fg(theme::muted_color())),
            Rect {
                x: area.x,
                y: banner_y,
                width: area.width,
                height: 1,
            },
        );
    } else if let HomeScreenMode::WindowsInBackground { total, minimized } = mode {
        let banner = if minimized == total {
            format!(" {total} window(s) minimized — bottom bar or {WINDOW_FOCUS_HINT} to restore ")
        } else {
            format!(" {total} window(s) open — bottom bar or {WINDOW_FOCUS_HINT} to focus ")
        };
        frame.render_widget(
            Paragraph::new(banner).style(Style::default().fg(theme::muted_color())),
            Rect {
                x: area.x,
                y: banner_y,
                width: area.width,
                height: 1,
            },
        );
    }

    render_grid(
        frame,
        layout.grid,
        GridView {
            entries: &list,
            selected: home.selected,
            scroll_row: home.scroll_row,
            columns: layout.columns,
            visible_rows: layout.tile_rows_visible,
        },
        interactions,
    );

    let footer = format!(
        " ⊞ Palette {LAUNCHER_SHORTCUT_HINT} · {WORKSPACE_HINT} · ↑↓←→ · Enter open · letter shortcuts"
    );
    frame.render_widget(
        Paragraph::new(footer).style(Style::default().fg(theme::muted_color())),
        Rect {
            x: area.x,
            y: area.y + area.height.saturating_sub(1),
            width: area.width,
            height: 1,
        },
    );
}

struct GridView<'a> {
    entries: &'a [HomeEntry],
    selected: usize,
    scroll_row: usize,
    columns: usize,
    visible_rows: usize,
}

fn render_grid(
    frame: &mut Frame,
    area: Rect,
    view: GridView<'_>,
    interactions: &mut InteractionMap,
) {
    if view.entries.is_empty() || area.width == 0 || area.height == 0 {
        return;
    }
    let tile_width = (area.width / view.columns as u16).max(TILE_MIN_WIDTH);
    let row_stride = TILE_HEIGHT + TILE_GAP;

    for vis_row in 0..view.visible_rows {
        let row = view.scroll_row + vis_row;
        for col in 0..view.columns {
            let index = row * view.columns + col;
            if index >= view.entries.len() {
                continue;
            }
            let entry = &view.entries[index];
            let x = area.x + col as u16 * tile_width;
            let y = area.y + vis_row as u16 * row_stride;
            if y + TILE_HEIGHT > area.y + area.height {
                break;
            }
            let rect = Rect {
                x,
                y,
                width: tile_width.saturating_sub(TILE_GAP),
                height: TILE_HEIGHT,
            };
            let focused = index == view.selected;
            render_tile(frame, rect, entry, focused);
            interactions.register(InteractionLayer::Content, rect, open_action(entry.kind));
        }
    }
}

fn render_tile(frame: &mut Frame, area: Rect, entry: &HomeEntry, focused: bool) {
    let shortcut = entry
        .shortcut
        .map(|ch| format!("[{ch}] "))
        .unwrap_or_else(|| "    ".to_owned());
    let pin = if entry.pinned { "★ " } else { "  " };
    let title = format!("{pin}{shortcut}{}", entry.title);
    let detail = truncate(&entry.description, area.width.saturating_sub(2) as usize);
    let border = if focused {
        theme::focused_border()
    } else {
        theme::unfocused_border()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border)
        .style(
            Style::default()
                .bg(theme::surface())
                .fg(theme::text())
                .add_modifier(if focused {
                    Modifier::REVERSED
                } else {
                    Modifier::empty()
                }),
        );
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new(format!("{title}\n{detail}")).style(Style::default().fg(theme::text())),
        inner,
    );
}

fn truncate(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        return text.to_owned();
    }
    if max_len <= 1 {
        return "…".to_owned();
    }
    format!(
        "{}…",
        text.chars()
            .take(max_len.saturating_sub(1))
            .collect::<String>()
    )
}
