use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::AppState;

use super::{geometry::UiGeometry, theme};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, geometry: &UiGeometry) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(13),
            Constraint::Min(20),
            Constraint::Length(24),
        ])
        .split(area);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" TDE ", theme::active()),
            Span::styled("  ·  desktop", Style::default().fg(theme::TEXT)),
        ])),
        chunks[0],
    );

    let mut workspace_spans = Vec::new();
    for index in 0..state.workspaces.len() {
        let style = if index == state.active_workspace {
            theme::active()
        } else {
            Style::default().fg(theme::MUTED)
        };
        workspace_spans.push(Span::styled(format!("  {}  ", index + 1), style));
    }
    frame.render_widget(
        Paragraph::new(Line::from(workspace_spans)).alignment(Alignment::Center),
        chunks[1],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("● ", Style::default().fg(theme::GREEN)),
            Span::styled(&state.machine.name, Style::default().fg(theme::TEXT)),
        ]))
        .alignment(Alignment::Right),
        chunks[2],
    );

    let _ = geometry;
}
