use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Paragraph, Row, Table},
};

use crate::app::{
    settings::{SettingsRow, SettingsState, display_value},
    state::AppState,
};

use super::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, view: &SettingsState) {
    let config = &state.config;
    let rows: Vec<Row> = SettingsRow::ALL
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let selected = index == view.selected;
            let label = row.label();
            let value = display_value(config, view, *row);
            let style = if selected {
                Style::default()
                    .fg(theme::text())
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default().fg(theme::text())
            };
            Row::new(vec![label.to_owned(), value]).style(style)
        })
        .collect();

    let help = "↑↓ move · Enter toggle/cycle theme · +/- refresh · s save";
    if area.height < 3 {
        frame.render_widget(Paragraph::new(help).style(theme::muted()), area);
        return;
    }
    let chunks = ratatui::layout::Layout::vertical([
        ratatui::layout::Constraint::Length(1),
        ratatui::layout::Constraint::Min(1),
    ])
    .split(area);
    frame.render_widget(Paragraph::new(help).style(theme::muted()), chunks[0]);
    frame.render_widget(
        Table::new(
            rows,
            [
                ratatui::layout::Constraint::Length(28),
                ratatui::layout::Constraint::Min(20),
            ],
        )
        .column_spacing(2),
        chunks[1],
    );
}
