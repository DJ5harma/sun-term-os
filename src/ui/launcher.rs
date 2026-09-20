use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem},
};

use crate::app::{AppState, ApplicationKind};

use super::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    frame.render_widget(Clear, area);
    let items = ApplicationKind::ALL
        .into_iter()
        .enumerate()
        .map(|(index, application)| {
            let selected = index == state.launcher_selection;
            let style = if selected {
                theme::active()
            } else {
                Style::default().fg(theme::TEXT)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("  {}  ", application.title()), style),
                Span::styled(
                    application.launcher_description(),
                    if selected { style } else { theme::muted() },
                ),
            ]))
        });
    frame.render_widget(
        List::new(items).block(
            Block::default()
                .title(" APPLICATION LAUNCHER ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::AMBER))
                .style(Style::default().bg(theme::SURFACE)),
        ),
        area,
    );
}
