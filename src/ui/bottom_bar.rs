use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    actions::Action,
    app::{AppState, Loadable},
    input::shell_shortcuts::LAUNCHER_SHORTCUT_HINT,
};

use super::{hit_map::HitMap, theme};

pub fn columns(area: Rect) -> [Rect; 3] {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(26),
            Constraint::Min(12),
            Constraint::Length(28),
        ])
        .split(area);
    [chunks[0], chunks[1], chunks[2]]
}

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, hits: &mut HitMap) {
    let [launcher, windows, status] = columns(area);
    hits.register(launcher, Action::ToggleLauncher);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" ⊞ ", Style::default().fg(theme::BG).bg(theme::AMBER)),
            Span::styled("Apps", Style::default().fg(theme::TEXT)),
            Span::styled(
                format!(" {LAUNCHER_SHORTCUT_HINT}"),
                Style::default().fg(theme::MUTED),
            ),
        ])),
        launcher,
    );

    let mut window_spans = Vec::new();
    for window in &state.current_workspace().windows {
        let active = Some(window.id) == state.current_workspace().focused_window;
        window_spans.push(Span::styled(
            format!(" {} ", window.application.title()),
            if active {
                theme::active()
            } else {
                Style::default().fg(theme::MUTED)
            },
        ));
    }
    if window_spans.is_empty() {
        window_spans.push(Span::styled(" — ", theme::muted()));
    }
    frame.render_widget(Paragraph::new(Line::from(window_spans)), windows);
    let workspace_windows = &state.current_workspace().windows;
    if !workspace_windows.is_empty() {
        let cells = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(std::iter::repeat_n(
                Constraint::Length(20),
                workspace_windows.len(),
            ))
            .split(windows);
        for (window, cell) in workspace_windows.iter().zip(cells.iter()) {
            hits.register(*cell, Action::FocusWindow(window.id));
        }
    }

    let process_count = match &state.processes {
        Loadable::Ready(processes) => format!("{} proc", processes.len()),
        Loadable::Loading => "proc…".to_owned(),
        Loadable::Failed(_) => "proc —".to_owned(),
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(process_count, Style::default().fg(theme::BLUE)),
            Span::styled(" · ", theme::muted()),
            Span::styled(state.status.as_str(), theme::muted()),
        ]))
        .alignment(Alignment::Right),
        status,
    );
}
