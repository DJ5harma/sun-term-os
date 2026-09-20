use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};

use crate::{
    actions::{Action, PaletteAction, ShellAction},
    app::{AppState, palette::filtered_entries},
};

#[derive(Debug, Clone, Copy)]
pub struct TopBarGeometry {
    pub area: Rect,
    pub brand: Rect,
    pub workspaces: Rect,
    pub machine: Rect,
}

#[derive(Debug, Clone, Copy)]
pub struct BottomBarGeometry {
    pub area: Rect,
    pub show_desktop: Rect,
    pub launcher: Rect,
    pub windows: Rect,
    pub status: Rect,
}

#[derive(Debug, Clone, Copy)]
pub struct UiGeometry {
    pub top_bar: TopBarGeometry,
    pub desktop: Rect,
    pub bottom_bar: BottomBarGeometry,
    pub launcher: Rect,
}

impl Default for UiGeometry {
    fn default() -> Self {
        calculate(Rect::new(0, 0, 80, 24), &AppState::default())
    }
}

pub const TOP_BAR_HEIGHT: u16 = 1;
pub const BOTTOM_BAR_HEIGHT: u16 = 1;

pub fn calculate(area: Rect, _state: &AppState) -> UiGeometry {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(TOP_BAR_HEIGHT),
            Constraint::Min(5),
            Constraint::Length(BOTTOM_BAR_HEIGHT),
        ])
        .split(area);
    UiGeometry {
        top_bar: top_bar_layout(root[0]),
        desktop: root[1],
        bottom_bar: bottom_bar_layout(root[2]),
        launcher: launcher_palette_rect(area, _state),
    }
}

/// Height fits the command list (no huge empty modal); width stays readable.
pub fn launcher_palette_rect(area: Rect, state: &AppState) -> Rect {
    const MAX_LIST_ROWS: usize = 18;
    const MIN_LIST_ROWS: usize = 4;
    let entry_count = if state.launcher_open {
        filtered_entries(state).len()
    } else {
        MIN_LIST_ROWS
    };
    let list_rows = if entry_count == 0 {
        MIN_LIST_ROWS
    } else {
        entry_count.clamp(MIN_LIST_ROWS, MAX_LIST_ROWS)
    };
    // Block top+bottom border, query line, one line per result.
    let height = 2 + 1 + list_rows as u16;
    let percent_y = (height * 100 / area.height.max(1)).clamp(14, 55);
    centered_rect(64, percent_y, area)
}

pub fn top_bar_layout(area: Rect) -> TopBarGeometry {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(18),
            Constraint::Min(20),
            Constraint::Length(24),
        ])
        .split(area);
    TopBarGeometry {
        area,
        brand: chunks[0],
        workspaces: chunks[1],
        machine: chunks[2],
    }
}

pub fn bottom_bar_layout(area: Rect) -> BottomBarGeometry {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(48),
            Constraint::Min(12),
            Constraint::Length(28),
        ])
        .split(area);
    let left = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(10)])
        .split(chunks[0]);
    BottomBarGeometry {
        area,
        show_desktop: left[0],
        launcher: left[1],
        windows: chunks[1],
        status: chunks[2],
    }
}

pub fn workspace_cells(workspaces_area: Rect, count: usize) -> Vec<(usize, Rect)> {
    if count == 0 {
        return Vec::new();
    }
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(std::iter::repeat_n(
            Constraint::Ratio(1, count as u32),
            count,
        ))
        .split(workspaces_area)
        .iter()
        .copied()
        .enumerate()
        .collect()
}

pub fn window_tab_cells(windows_area: Rect, count: usize) -> Vec<Rect> {
    if count == 0 {
        return Vec::new();
    }
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(std::iter::repeat_n(
            Constraint::Ratio(1, count as u32),
            count,
        ))
        .split(windows_area)
        .to_vec()
}

/// Chrome bars use layout math only — same cells as render, no InteractionMap registration.
pub fn shell_action_at(
    column: u16,
    row: u16,
    geometry: &UiGeometry,
    state: &AppState,
) -> Option<Action> {
    let position = Position::new(column, row);

    if geometry.bottom_bar.area.contains(position) {
        if geometry.bottom_bar.show_desktop.contains(position) {
            return Some(Action::Shell(ShellAction::ToggleShowDesktop));
        }
        if geometry.bottom_bar.launcher.contains(position) {
            return Some(Action::Palette(PaletteAction::ToggleLauncher));
        }
        let workspace_windows = &state.current_workspace().windows;
        for (window, cell) in workspace_windows.iter().zip(window_tab_cells(
            geometry.bottom_bar.windows,
            workspace_windows.len(),
        )) {
            if cell.contains(position) {
                return Some(Action::Shell(ShellAction::FocusWindow(window.id)));
            }
        }
        return None;
    }

    if geometry.top_bar.area.contains(position) {
        for (index, cell) in workspace_cells(geometry.top_bar.workspaces, state.workspaces.len()) {
            if cell.contains(position) {
                return Some(Action::Shell(ShellAction::SwitchWorkspace(index)));
            }
        }
    }

    None
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_has_stable_top_and_bottom_bars() {
        let state = AppState::default();
        let geometry = calculate(Rect::new(0, 0, 120, 40), &state);
        assert_eq!(geometry.top_bar.area.height, TOP_BAR_HEIGHT);
        assert_eq!(geometry.bottom_bar.area.height, BOTTOM_BAR_HEIGHT);
    }

    #[test]
    fn shell_action_matches_bottom_bar_cells() {
        use crate::domain::{ApplicationKind, Window, WindowState};
        use crate::machine::MachineId;
        let mut state = AppState::default();
        let workspace = &mut state.workspaces[0];
        workspace.windows.push(Window {
            id: 1,
            application: ApplicationKind::Terminal,
            state: WindowState::Normal,
            machine_id: MachineId::Local,
        });
        workspace.focused_window = Some(1);
        let geometry = calculate(Rect::new(0, 0, 100, 24), &state);
        let desktop_x = geometry.bottom_bar.show_desktop.x + 2;
        assert_eq!(
            shell_action_at(
                desktop_x,
                geometry.bottom_bar.show_desktop.y,
                &geometry,
                &state
            ),
            Some(Action::Shell(ShellAction::ToggleShowDesktop))
        );
        let launcher_x = geometry.bottom_bar.launcher.x + 2;
        assert_eq!(
            shell_action_at(
                launcher_x,
                geometry.bottom_bar.launcher.y,
                &geometry,
                &state
            ),
            Some(Action::Palette(PaletteAction::ToggleLauncher))
        );
        let windows = &state.current_workspace().windows;
        let cell = window_tab_cells(geometry.bottom_bar.windows, windows.len())[0];
        assert_eq!(
            shell_action_at(cell.x + cell.width / 2, cell.y, &geometry, &state),
            Some(Action::Shell(ShellAction::FocusWindow(windows[0].id)))
        );
    }
}
