use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{actions::Action, app::AppState};

use super::{hit_map::HitMap, theme};

/// Top bar column layout shared with hit-testing in [geometry].
pub fn columns(area: Rect) -> [Rect; 3] {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(14),
            Constraint::Min(20),
            Constraint::Length(24),
        ])
        .split(area);
    [chunks[0], chunks[1], chunks[2]]
}

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, hits: &mut HitMap) {
    let [brand, workspaces, machine] = columns(area);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" TDE ", theme::active()),
            Span::styled("desktop", Style::default().fg(theme::MUTED)),
        ])),
        brand,
    );

    for (index, cell) in super::geometry::workspace_cells(workspaces, state.workspaces.len()) {
        hits.register(cell, Action::SwitchWorkspace(index));
        let label = format!(" F{} ", index + 1);
        let line = if index == state.active_workspace {
            Line::from(Span::styled(label, theme::active()))
        } else {
            Line::from(Span::styled(label, Style::default().fg(theme::MUTED)))
        };
        frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), cell);
    }
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("● ", Style::default().fg(theme::GREEN)),
            Span::styled(&state.machine.name, Style::default().fg(theme::TEXT)),
        ]))
        .alignment(Alignment::Right),
        machine,
    );
}
