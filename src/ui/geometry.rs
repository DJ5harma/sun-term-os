use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::{
    app::AppState,
    domain::{ApplicationKind, WindowId},
};

#[derive(Debug, Clone, Default)]
pub struct UiGeometry {
    pub top_bar: Rect,
    pub desktop: Rect,
    pub bottom_bar: Rect,
    pub launcher: Rect,
    pub launcher_button: Rect,
    pub launcher_targets: Vec<(ApplicationKind, Rect)>,
    pub workspace_targets: Vec<(usize, Rect)>,
    pub window_targets: Vec<(WindowId, Rect)>,
}

pub fn calculate(area: Rect, state: &AppState) -> UiGeometry {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(13),
            Constraint::Min(20),
            Constraint::Length(24),
        ])
        .split(root[0]);
    let workspace_targets = workspace_targets(top[1], state.workspaces.len());
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(13), Constraint::Min(12)])
        .split(root[2]);
    let window_targets = window_targets(bottom[1], state);
    let launcher = centered_rect(64, 62, area);
    UiGeometry {
        top_bar: root[0],
        desktop: root[1],
        bottom_bar: root[2],
        launcher,
        launcher_button: top[0],
        launcher_targets: launcher_targets(launcher),
        workspace_targets,
        window_targets,
    }
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

fn workspace_targets(area: Rect, count: usize) -> Vec<(usize, Rect)> {
    if count == 0 {
        return Vec::new();
    }
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(std::iter::repeat_n(Constraint::Length(5), count))
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
        assert_eq!(geometry.top_bar.height, 3);
        assert_eq!(geometry.bottom_bar.height, 3);
        assert_eq!(geometry.workspace_targets.len(), 3);
    }
}
