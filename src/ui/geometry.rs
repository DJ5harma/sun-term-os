use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::{
    app::AppState,
    domain::{ApplicationKind, WindowId},
    ui::{bottom_bar, file_manager, top_bar, windows},
};

#[derive(Debug, Clone, Default)]
pub struct UiGeometry {
    pub top_bar: Rect,
    pub desktop: Rect,
    pub bottom_bar: Rect,
    pub launcher: Rect,
    pub launcher_button: Rect,
    pub launcher_targets: Vec<(ApplicationKind, Rect)>,
    pub window_targets: Vec<(WindowId, Rect)>,
    pub file_manager_hit_targets: Vec<(WindowId, file_manager::FileManagerHitTargets)>,
}

type FileManagerGeometryTargets = Vec<(WindowId, file_manager::FileManagerHitTargets)>;

/// Vertical chrome: top bar (workspace tabs), single-line bottom status strip.
pub const TOP_BAR_HEIGHT: u16 = 3;
pub const BOTTOM_BAR_HEIGHT: u16 = 1;

pub fn calculate(area: Rect, state: &AppState) -> UiGeometry {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(TOP_BAR_HEIGHT),
            Constraint::Min(5),
            Constraint::Length(BOTTOM_BAR_HEIGHT),
        ])
        .split(area);
    let top_columns = top_bar::columns(root[0]);
    let bottom_columns = bottom_bar::columns(root[2]);
    let window_targets = window_targets(bottom_columns[1], state);
    let launcher = centered_rect(64, 62, area);
    let file_manager_hit_targets = file_manager_targets(root[1], state);
    UiGeometry {
        top_bar: root[0],
        desktop: root[1],
        bottom_bar: root[2],
        launcher,
        launcher_button: top_columns[0],
        launcher_targets: launcher_targets(launcher),
        window_targets,
        file_manager_hit_targets,
    }
}

fn file_manager_targets(desktop: Rect, state: &AppState) -> FileManagerGeometryTargets {
    let Some(window) = state
        .focused_window()
        .filter(|window| window.application == ApplicationKind::FileManager)
    else {
        return Vec::new();
    };
    let Some(manager) = state.file_manager(window.id) else {
        return Vec::new();
    };
    let window_inner = windows::content_inner(desktop);
    let panel_inner = file_manager::panel_inner(window_inner);
    let layout = file_manager::layout(panel_inner);
    let targets = file_manager::hit_targets(&layout, manager);
    vec![(window.id, targets)]
}

fn launcher_targets(area: Rect) -> Vec<(ApplicationKind, Rect)> {
    let inner = Rect::new(
        area.x.saturating_add(1),
        area.y.saturating_add(1),
        area.width.saturating_sub(2),
        area.height.saturating_sub(2),
    );
    Layout::default()
        .direction(Direction::Vertical)
        .constraints(std::iter::repeat_n(
            Constraint::Length(1),
            ApplicationKind::ALL.len(),
        ))
        .split(inner)
        .iter()
        .copied()
        .zip(ApplicationKind::ALL)
        .map(|(rect, application)| (application, rect))
        .collect()
}

/// Workspace labels in the top bar; must match [top_bar::render] cell layout.
pub fn workspace_cells(area: Rect, count: usize) -> Vec<(usize, Rect)> {
    if count == 0 {
        return Vec::new();
    }
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(std::iter::repeat_n(
            Constraint::Ratio(1, count as u32),
            count,
        ))
        .split(area)
        .iter()
        .copied()
        .enumerate()
        .collect()
}

fn window_targets(area: Rect, state: &AppState) -> Vec<(WindowId, Rect)> {
    let windows = &state.current_workspace().windows;
    if windows.is_empty() {
        return Vec::new();
    }
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(std::iter::repeat_n(Constraint::Length(20), windows.len()))
        .split(area)
        .iter()
        .copied()
        .zip(windows.iter().map(|window| window.id))
        .map(|(rect, id)| (id, rect))
        .collect()
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
        assert_eq!(geometry.top_bar.height, TOP_BAR_HEIGHT);
        assert_eq!(geometry.bottom_bar.height, BOTTOM_BAR_HEIGHT);
    }
}
