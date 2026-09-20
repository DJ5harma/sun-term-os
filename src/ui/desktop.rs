use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{
    app::AppState,
    domain::WindowState,
    input::{WINDOW_FOCUS_HINT, WORKSPACE_HINT},
};

use super::{interaction::InteractionMap, theme, windows};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, interactions: &mut InteractionMap) {
    let workspace = state.current_workspace();
    if workspace.windows.is_empty() {
        let text = format!(
            "The desktop is ready.\n\n\
             Open the command palette (⊞ Apps or Alt+P) to search commands.\n\
             Press t for a terminal window.\n\
             Press f for the file manager · p for processes · s for system info.\n\
             Open Settings from the palette to change theme and behavior.\n\
             In the file manager, o opens the selection in a new terminal.\n\
             Focus windows: {WINDOW_FOCUS_HINT} (bar order) or click bar tabs · workspaces: {WORKSPACE_HINT}."
        );
        frame.render_widget(
            Paragraph::new(text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::SURFACE_ALT)),
                )
                .style(Style::default().fg(theme::TEXT))
                .wrap(Wrap { trim: true }),
            area,
        );
    } else if let Some(window) = state.focused_window() {
        windows::render(frame, area, state, window, interactions);
    } else if workspace
        .windows
        .iter()
        .any(|window| window.state == WindowState::Minimized)
    {
        let text = format!(
            "All windows are minimized.\n\n\
             Click a tab in the bottom bar or use {WINDOW_FOCUS_HINT} to restore one."
        );
        frame.render_widget(
            Paragraph::new(text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::SURFACE_ALT)),
                )
                .style(Style::default().fg(theme::TEXT))
                .wrap(Wrap { trim: true }),
            area,
        );
    }
}
