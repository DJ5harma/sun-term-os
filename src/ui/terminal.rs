use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::Paragraph,
};

use crate::app::{AppState, TerminalStatus};

use super::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, window_id: u64) {
    let view = state.terminal_view(window_id);
    let scroll_hint = if view.is_some_and(|view| !view.follow_output()) {
        " · Alt+PgDn follow"
    } else {
        ""
    };
    let status = match state.terminal_status(window_id) {
        Some(TerminalStatus::Starting) => format!("Starting shell…{scroll_hint}"),
        Some(TerminalStatus::Running) => {
            format!("Running · Alt+PgUp/PgDn scroll{scroll_hint}")
        }
        Some(TerminalStatus::Exited) => "Shell exited · Ctrl+W closes this window".to_owned(),
        Some(TerminalStatus::Failed(_)) => {
            "Shell failed to start · Ctrl+W closes this window".to_owned()
        }
        None => format!("Starting shell…{scroll_hint}"),
    };
    let status_style = match state.terminal_status(window_id) {
        Some(TerminalStatus::Exited | TerminalStatus::Failed(_)) => {
            Style::default().fg(theme::red())
        }
        _ => Style::default().fg(theme::muted_color()),
    };

    let visible_rows = area.height.saturating_sub(2).max(1) as usize;
    let content = if let Some(view) = view {
        view.visible_text(visible_rows)
    } else {
        state.terminal_content(window_id).to_owned()
    };

    if area.height < 2 {
        frame.render_widget(
            Paragraph::new(content).style(Style::default().fg(theme::text())),
            area,
        );
        return;
    }

    let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(area);
    frame.render_widget(
        Paragraph::new(content).style(Style::default().fg(theme::text())),
        chunks[0],
    );
    frame.render_widget(Paragraph::new(status).style(status_style), chunks[1]);
}
