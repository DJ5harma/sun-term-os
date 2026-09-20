use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::Paragraph,
};

use crate::app::{AppState, TerminalStatus};

use super::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, window_id: u64) {
    let content = state.terminal_content(window_id);
    let status = match state.terminal_status(window_id) {
        Some(TerminalStatus::Starting) => "Starting shell…",
        Some(TerminalStatus::Running) => "Running · Ctrl+C interrupts the foreground command",
        Some(TerminalStatus::Exited) => "Shell exited · Ctrl+W closes this window",
        Some(TerminalStatus::Failed(_)) => "Shell failed to start · Ctrl+W closes this window",
        None => "Starting shell…",
    };
    let status_style = match state.terminal_status(window_id) {
        Some(TerminalStatus::Exited | TerminalStatus::Failed(_)) => Style::default().fg(theme::RED),
        _ => Style::default().fg(theme::MUTED),
    };

    if area.height < 2 {
        frame.render_widget(
            Paragraph::new(content).style(Style::default().fg(theme::TEXT)),
            area,
        );
        return;
    }

    let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(area);
    frame.render_widget(
        Paragraph::new(content).style(Style::default().fg(theme::TEXT)),
        chunks[0],
    );
    frame.render_widget(Paragraph::new(status).style(status_style), chunks[1]);
}
