use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::{app::AppState, ui::bottom_bar};

#[derive(Debug, Clone, Default)]
pub struct UiGeometry {
    pub top_bar: Rect,
    pub desktop: Rect,
    pub bottom_bar: Rect,
    pub launcher: Rect,
}

/// Vertical chrome: top bar (workspace tabs), single-line bottom status strip.
pub const TOP_BAR_HEIGHT: u16 = 3;
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
    let _ = bottom_bar::columns(root[2]);
    let launcher = centered_rect(64, 62, area);
    UiGeometry {
        top_bar: root[0],
        desktop: root[1],
        bottom_bar: root[2],
        launcher,
    }
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
