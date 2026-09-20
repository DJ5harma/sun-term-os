use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::{AppState, Loadable};

use super::{geometry::UiGeometry, theme};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, geometry: &UiGeometry) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(13),
            Constraint::Min(20),
            Constraint::Length(32),
        ])
        .split(area);
    frame.render_widget(
        Paragraph::new("  running  ").style(Style::default().fg(theme::BG).bg(theme::AMBER)),
        chunks[0],
    );

    let mut windows = Vec::new();
    for window in &state.current_workspace().windows {
        let active = Some(window.id) == state.current_workspace().focused_window;
        windows.push(Span::styled(
            format!(" {}  ", window.application.title()),
            if active {
                theme::active()
            } else {
                Style::default().fg(theme::MUTED)
            },
        ));
    }
    if windows.is_empty() {
        windows.push(Span::styled(" no open applications ", theme::muted()));
    }
    frame.render_widget(Paragraph::new(Line::from(windows)), chunks[1]);

    let process_count = match &state.processes {
        Loadable::Ready(processes) => format!("{} processes", processes.len()),
        Loadable::Loading => "processes loading".to_owned(),
        Loadable::Failed(_) => "processes unavailable".to_owned(),
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(process_count, Style::default().fg(theme::BLUE)),
            Span::styled("  ·  ", theme::muted()),
            Span::styled(&state.status, theme::muted()),
        ]))
        .alignment(ratatui::layout::Alignment::Right),
        chunks[2],
    );
    let _ = geometry;
}
