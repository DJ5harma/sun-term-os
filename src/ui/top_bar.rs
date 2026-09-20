use ratatui::{
    Frame,
    layout::Alignment,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::AppState;

use super::{
    geometry::{TopBarGeometry, workspace_cells},
    theme,
};

pub fn render(frame: &mut Frame, layout: TopBarGeometry, state: &AppState) {
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" TDE ", theme::active()),
            Span::styled("desktop", Style::default().fg(theme::MUTED)),
        ])),
        layout.brand,
    );

    for (index, cell) in workspace_cells(layout.workspaces, state.workspaces.len()) {
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
            Span::styled(state.host_label(), Style::default().fg(theme::TEXT)),
        ]))
        .alignment(Alignment::Right),
        layout.machine,
    );
}
