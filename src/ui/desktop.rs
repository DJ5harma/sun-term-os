use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::AppState;

use super::{theme, windows};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let workspace = state.current_workspace();
    if workspace.windows.is_empty() {
        let text = [
            "The desktop is ready.",
            "",
            "Press p to launch an application.",
            "Press t for a terminal window.",
            "Press f for the file manager.",
            "Switch workspaces with F1, F2, or F3.",
        ];
        frame.render_widget(
            Paragraph::new(text.join("\n"))
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
        windows::render(frame, area, state, window);
    }
}
