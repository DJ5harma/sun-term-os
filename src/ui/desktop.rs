use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::AppState;

use super::{hit_map::HitMap, theme, windows};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, hits: &mut HitMap) {
    let workspace = state.current_workspace();
    if workspace.windows.is_empty() {
        let text = [
            "The desktop is ready.",
            "",
            "Open Apps from the bottom bar (⊞) or press Ctrl+Shift+P.",
            "Press t for a terminal window.",
            "Press f for the file manager.",
            "Switch workspaces: click F1–F3 above or press those keys.",
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
        windows::render(frame, area, state, window, hits);
    }
}
